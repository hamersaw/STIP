use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use comm::StreamHandler;
use gdal::Dataset;
use geocode::Geocode;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::album::AlbumManager;

use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpStream, SocketAddr};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(FromPrimitive)]
enum TransferOp {
    ReadImage = 0,
    WriteImage = 1,
}

pub struct TransferStreamHandler {
    album_manager: Arc<RwLock<AlbumManager>>,
}

impl TransferStreamHandler {
    pub fn new(album_manager: Arc<RwLock<AlbumManager>>)
            -> TransferStreamHandler {
        TransferStreamHandler {
            album_manager: album_manager,
        }
    }
}

impl StreamHandler for TransferStreamHandler {
    fn process(&self, stream: &mut TcpStream)
            -> Result<(), Box<dyn Error>> {
        // read operation type
        let op_type = stream.read_u8()?;
        match FromPrimitive::from_u8(op_type) {
            Some(TransferOp::ReadImage) => {
                // read path
                let path_string = read_string(stream)?;
                let path = PathBuf::from(&path_string);

                if !path.exists() {
                    stream.write_u8(1)?;
                    write_string(&format!("path '{}' does not exist",
                        path_string), stream)?;
                    return Ok(());
                }

                // open dataset
                let dataset = match Dataset::open(&path) {
                    Ok(dataset) => dataset,
                    Err(e) => {
                        stream.write_u8(1)?;
                        write_string(&e.to_string(), stream)?;
                        return Ok(());
                    },
                };

                // if exists -> split dataset to subgeocode
                let subgeocode_indicator = stream.read_u8()?;
                if subgeocode_indicator == 0 {
                    // no need to split image -> write image
                    stream.write_u8(0)?;
                    st_image::serialize::write(&dataset, stream)?;
                } else {
                    // read subgeocode metadata
                    let geocode_value = stream.read_u8()?;
                    let geocode: Geocode = match geocode_value {
                        0 => Geocode::Geohash,
                        1 => Geocode::QuadTile,
                        _ => {
                            let err_msg = format!("unknown geocode {}",
                                geocode_value);
                            stream.write_u8(1)?;
                            write_string(&err_msg, stream)?;
                            return Err(err_msg.into());
                        },
                    };

                    let subgeocode = read_string(stream)?;

                    // split image with geocode precision
                    let precision = subgeocode.len();

                    // compute geohash window boundaries for dataset
                    let epsg_code = geocode.get_epsg_code();
                    let (x_interval, y_interval) =
                        geocode.get_intervals(precision);

                    let (image_min_cx, image_max_cx, 
                            image_min_cy, image_max_cy) =
                        st_image::coordinate::get_bounds(
                            &dataset, epsg_code)?;

                    let window_bounds = 
                        st_image::coordinate::get_windows(image_min_cx,
                            image_max_cx, image_min_cy, image_max_cy,
                            x_interval, y_interval);

                    // iterate over window bounds
                    for (min_cx, max_cx, min_cy, max_cy) in 
                            window_bounds {
                        // perform dataset split
                        let split_dataset = match 
                                st_image::transform::split(&dataset,
                                    min_cx, max_cx, min_cy, max_cy, 
                                    epsg_code)? {
                            Some(split_dataset) => split_dataset,
                            None => continue,
                        };

                        let split_geocode = geocode.encode(
                            (min_cx + max_cx) / 2.0,
                            (min_cy + max_cy) / 2.0, precision)?;

                        // check if this is the desired geocode
                        if split_geocode.to_lowercase()
                                != subgeocode.to_lowercase() {
                            continue;
                        }

                        // process valid subdataset
                        stream.write_u8(0)?;
                        st_image::serialize::write(
                            &split_dataset, stream)?;
                        return Ok(())
                    }

                    // failed to split image into subgeocode
                    stream.write_u8(1)?;
                    write_string(&format!(
                        "failed to split image into geocode '{}'",
                            subgeocode), stream)?;
                    return Ok(());
                }
            },
            Some(TransferOp::WriteImage) => {
                // read everything
                let album = read_string(stream)?;
                let mut dataset = st_image::serialize::read(stream)?;
                let geocode = read_string(stream)?;
                let pixel_coverage = stream.read_f64::<BigEndian>()?;
                let platform = read_string(stream)?;
                let source = read_string(stream)?;
                let subdataset = stream.read_u8()?;
                let tile = read_string(stream)?;
                let timestamp = stream.read_i64::<BigEndian>()?;

                // write image using AlbumManager
                let album_manager = self.album_manager.read().unwrap();
                match album_manager.get(&album) {
                    Some(album) => {
                        let mut album = album.write().unwrap();
                        album.write(&mut dataset, &geocode,
                            pixel_coverage, &platform, &source,
                            subdataset, &tile, timestamp)?;
                    },
                    None => warn!("album '{}' does not exist", album),
                }

                // write success
                stream.write_u8(1)?;
            },
            None => return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("unsupported operation type '{}'", op_type)))),
        }

        Ok(())
    }
}

pub fn read_string<T: Read>(reader: &mut T)
        -> Result<String, Box<dyn Error>> {
    let len = reader.read_u8()?;
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf)?;
    Ok(String::from_utf8(buf)?)
}

pub fn send_image(addr: &SocketAddr, album: &str, dataset: &Dataset,
        geocode: &str, pixel_coverage: f64, platform: &str,
        source: &str, subdataset: u8, tile: &str, timestamp: i64)
        -> Result<(), Box<dyn Error>> {
    // open connection
    let mut stream = TcpStream::connect(addr)?;
    stream.write_u8(TransferOp::WriteImage as u8)?;

    // write everything
    write_string(&album, &mut stream)?;
    st_image::serialize::write(&dataset, &mut stream)?;
    write_string(&geocode, &mut stream)?;
    stream.write_f64::<BigEndian>(pixel_coverage)?;
    write_string(&platform, &mut stream)?;
    write_string(&source, &mut stream)?;
    stream.write_u8(subdataset)?;
    write_string(&tile, &mut stream)?;
    stream.write_i64::<BigEndian>(timestamp)?;
 
    // read success
    let _ = stream.read_u8()?;

    Ok(())
}

pub fn write_string<T: Write>(value: &str, writer: &mut T)
        -> Result<(), Box<dyn Error>> {
    writer.write_u8(value.len() as u8)?;
    writer.write(value.as_bytes())?;
    Ok(())
}
