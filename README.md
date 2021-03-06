# stip (SpatioTemporal Image Partitioner)
## OVERVIEW
A distributed spatiotemporal image management framework.

## DOWNLOADING DATASETS
#### NASA Earthdata
[Earthdata](https://lpdaac.usgs.gov/tools/earthdata-search/) is a data search tool run by NASA. There is a variety of data. Downloading data may be acheived by using the provided download scripts on data searches.
#### Northwest Knowledge Network
The [Northwest Knowledge Network](https://www.northwestknowledge.net) is run by the University of Idaho. Downloading the permanent archive can be done by the command line

    wget --recursive --no-parent https://www.northwestknowledge.net/metdata/data/permanent/

#### USGS Earth Explorer
[Earth Explorer](https://earthexplorer.usgs.gov/) provides a variery of datasets and is our first stop in downloading new data. The process to download data is as follows:
1. Use the WebUI to download 'Comma Delmited' or 'CSV' data formats.
2. Extract the 'Display ID' field to a new file.
3. Submit to [Earth Explorer Bulk Order](https://earthexplorer.usgs.gov/filelist) interface using 'CSV Metadata Export' type.
4. Use the [Earth Explorer Bulk Data Application](https://www.usgs.gov/media/images/earthexplorer-bulk-download-application-bda) to download tiles.

## DATASETS
#### gridMET
GridMET data in NetCDF format is downloaded from the [Northwest Knowledge Network](#Northwest-Knowledge-Network).

Subdataset | Resolution | Data Type | Bands
---------- | ---------- | --------- | -----
0          | ~4km       | f32       | Max Temperature, Min Temperature, Max Humidity, Min Humidity, Specific Humidity, Wind Speed, Precipitation, Wind Direction, Shortwave Flux, Evapotranspiration Grass, Energy Release, Burning Index, Dead Fuel Moisture 100hr, Dead Fuel Moisture 1000hr, Evapotranspiration Alfalfa, Vapor Pressure Deficit
#### LANDSAT8 - Landsat8C1L1
Collection 1 Level 1 Landsat 8 data is located at 'Landsat/Landsat Collection 1/Landsat Collection 1 Level-1/Landsat 8 OLI/TIRS C1 Level-1' in [Earth Explorer](#USGS-Earth-Explorer). Loading it requires a pre-processing phase by executing a conversion script located at 'sbin/conversion/landsat8c1l1.sh' in this repository and using the 'generic' data loader in the [image store command](#IMAGE-STORE).

Subdataset | Resolution | Data Type | Bands
---------- | ---------- | --------- | -----
0          | 30m        | uint16    | B1, B2, B3, B4, B5, B6, B7 B9, B10, B11, QA
1          | 15m        | uint16    | B8
#### MODIS - MCD43A4
MCD43A4 is a MODIS product presented in hdf format located at 'NASA/LPDAAC Collections/MODIS BRDF and Albedo - V6/MCD43A4 V6' in [Earth Explorer](#USGS-Earth-Explorer).

Subdataset | Resolution | Data Type | Bands
---------- | ---------- | --------- | -----
0          | ~500m      | u8        | BRDF Albedo Quality 1, 2, 3, 4, 5, 6, 7
1          | ~500m      | int16     | NADIR Reflectance 1, 2, 3, 4, 5, 6, 7
#### MODIS - MOD11A1
The MODIS MOD11A1 in hdf format is located at 'NASA/LPDAAC Collections/MODIS Land Surface Temp and Emiss - V6/MODIS MOD11A1 V6' in [Earth Explorer](#USGS-Earth-Explorer). 

Subdataset | Resolution | Data Type | Bands
---------- | ---------- | --------- | -----
0          | 1km        | u8        | Day LST Quality, Day View Time, Day View Angle, Night LST Quality, Night View Time, Night View Angle, Band 31 Emissivity, Band 32 Imissivity
1          | 1km        | uint16    | Day LST, Night LST, Day Clear Sky-Coverage, Night Clear Sky-Coverage
#### MODIS - MOD11A2
MCD43A4 is a MODIS product presented in hdf format located at ''NASA/LPDAAC Collections/MODIS Land Surface Temp and Emiss - V6/MODIS MOD11A2 V6'' in [Earth Explorer](#USGS-Earth-Explorer).

Subdataset | Resolution | Data Type | Bands
---------- | ---------- | --------- | -----
0          | 1km        | u8        | Day LST Quality, Day View Time, Day View Angle, Night LST Quality, Night View Time, Night View Angle, Band 31 Emissivity, Band 32 Imissivity, Day Clear Sky-Coverage, Night Clear Sky-Coverage
1          | 1km        | uint16    | Day LST, Night LST
#### NAIP
NAIP in ZIP format (internally a single GeoTIFF image) is retreived using the 'Aerial Imagery/NAIP' in [Earth Explorer](#USGS-Earth-Explorer).

Subdataset | Resolution | Data Type | Bands
---------- | ---------- | --------- | -----
0          | 1m         | u8        | Red, Green, Blue, NIR
#### NLCD
The [National Land Cover Database](https://www.mrlc.gov/) may be processed in an IMG formated image.

Subdataset | Resolution | Data Type | Bands
---------- | ---------- | --------- | -----
0          | 30m        | u8        | Pixel Classification
#### Sentinel-2
Sentinel-2 data is processed using the SAFE format located at 'Sentinel/Sentinel-2' in [Earth Explorer](#USGS-Earth-Explorer).

Subdataset | Resolution | Data Type | Bands
---------- | ---------- | --------- | -----
0          | 10m        | uint16    | Blue, Green, Red, NIR
1          | 20m        | uint16    | Vegetation Red Index 1 & 2 & 3, Narrow NIR, SWIR 1 & 2
2          | 60m        | uint16    | Coastal Aerosol, Water Vapour, SWIR-Cirrus
3          | 10m        | u8        | TCI-R, TCI-G, TCI-B

## WORKSPACE
The implementation is structured using rust's workspace paradigm within the ./impl directory in the project root.
#### PROTOBUF
This project uses [gRPC](https://grpc.io/) and [Protocol Buffers](https://developers.google.com/protocol-buffers/) to present a language agnostic RPC interface. This paradigm is employed for all system communication (except data transfers). The protobuf rust crate includes protobuf compilation instructions along with project module export definitions.
#### STIP
This is the command line application for interfacing with the stip cluster. It includes a variety of testing and operational functionality explored further in the [COMMANDS](#COMMANDS) section below.
#### STIPD
This crate defines a stip node. It contains the bulk of the implementation; defining image partioning and distribution strategies and metadata queries among other functionality.

## COMMANDS
### STIPD
#### START CLUSTER
The cluster deployment is provided in the ./etc/hosts.txt file, where each row defines a single cluster node. Each row is formatted as 'ip_address gossip_port rpc_port xfer_portFLAGS...'. An example row is:
 
    127.0.0.1 15605 15606 15607 -d /tmp/STIP/0 -t 0 -t 6148914691236516864 -t 12297829382473033728

This row defines a stipd node running at the provided IP address (127.0.0.1) and ports (15605 15606 15607). Additionally it defines a variety of command line arguments including: -d <directory> to define the image storage directory and -t <token> to initialize this node with the provided DHT tokens. Generation of a multi-token dht can be performed using the provided script.

    # generate dht for 50 nodes with 3 tokens each
    ./sbin/generate-tokens.py 50 3

Starting the cluster leverages the provided ./sbin/start-all.sh script. This script simply iterates over nodes defined in ./etc/hosts.txt and starts a node instance on the provided machine. It should be noted that starting nodes on remote hosts requires ssh access.

    # terminal command to start stip cluster from root project
    ./sbin/start-all.sh
#### STOP CLUSTER
Similar to starting the cluster, the ./sbin/stop-all.sh script has been provided to stop a stip cluster. Again, this script leverages the ./etc/hosts.txt file to iterate over node definitions.

    # terminal command to stop stip cluster from root project
    ./sbin/stop-all.sh
### STIP
#### NODE LIST
This command is useful for identifying nodes within the cluster. It is typically used for testing or in the background of APIs or applications when contacting each cluster node is necessary for a particular operation.

    # list all nodes in the cluser
    ./stip node list
#### TASK LIST / CLEAR
Behind the scenes of stip all functionality is partitioned into a variety of tasks. Said functionality includes image loading, image splitting / merging, image filling, etc. The 'task' interface is used to monitor progress of cluster tasks.
    
    # list all cluster tasks
    ./stip task list

    # clear complete cluster tasks
    ./stip task clear
#### ALBUM CREATE
The system uses albums logically partition the dataspace. Each album is established using a unique identifier. Additionally, they define both the geocode algorithm and DHT key length for all images stored within. The geohash and quadtile geocode algorithms are currently supported. DHT key lengths which are positive use the first 'n' characters of the geocode, negative using geocode length - 'n' characters, and 0 uses the entire geocode.

    # create an album using quadtiles and distribute using the entire geocode
    ./stip album create test quadtile
    
    # distribute using the first two characters of geocode
    ./stip album create test2 geohash -d 2

    # distributed using geocode length - 1 characters of geocode
    ./stip album create test3 quadtile -d=-1
#### ALBUM LIST
This command lists available albums, including a variety of metadata.

    # list albums
    ./stip album list
#### ALBUM OPEN / CLOSE
Albums may be open and closed. Internally, the difference defines whether a in-memory index is mainained over the underlying dataspace. Externally, it determines whether an album may be queried or not. Since images are written to a directory, they may be written to an album regardless of whether it is open or closed. 

    # open an album
    ./stip album open test2

    # close an album
    ./stip album close test2
#### IMAGE STORE
Image tore tasks are initialized on a per-node basis, meaning **each node ony processes local data**. Therefore, data is typically distributed among cluster nodes to enable distributed processing. As such, a separate task must be manually started on each node to load the local data. Additionally, it must be stated that **the netCDF linux driver does not support multi-threading**. So any dataset in netCDF format must be loaded using a single thread.

    # store a single modis file into the test album at geohash length 3
    ./stip image store test '~/Downloads/earth-explorer/modis/MCD43A4.A2020100.h08v05.006.2020109032339.hdf' mcd43a4 -t 1 -l 3

    # store images for the given glob with 4 threads at geohash length 6
    ./stip image store test2 '~/Downloads/earth-explorer/naip/test/*' naip -t 4 -l 6

    # store sentinel data for files with the provided glob at geohash
    #   length 5 using 2 threads and setting the task id as 1000
    ./stip -i $(curl ifconfig.me) image store test3 "/s/$(hostname)/a/nobackup/galileo/usgs-earth-explorer/sentinel-2/foco-20km/*T13TEE*" sentinel2 -t 2 -l 5 -d 1000
#### IMAGE LIST / SEARCH
These commands enable searching the system for images using the metadata provided. 'image search' provides an agglomerated data representation, presenting image geohash precision counts satisfying the query. It is useful for gaining understanding of the dataspace. With an understanding of interesting data the 'image list' command returns all metadata for images satisfying the provided filtering criteria.

    # search for NAIP data in the test album where the geohash starts with '9x'
    ./stip image search test -p NAIP -g 9x -r 

    # search for data beginning on 2015-01-01 within the test2 album
    #   where the pixel coverage is greater than 95%
    ./stip image search test2 -s 2524608000 -x 0.95

    # list all images from Sentinel-2 dataset for geohash '9xj3ej'
    ./stip image list test3 -p Sentinel-2 -g 9xj3ej
#### IMAGE SPLIT
Images are stored at the geohash length defined during 'image store's. However, the 'image split' command enables further partitioning of datasets. This command launches a task on each cluster node to process data local to that machine. This command employs many of the same filtering criteria as 'image search' and 'image list' commands, enabling fine image processing filtering criteria.

    # split Sentinel-2 data at a geohash length of 6 
    #   for all geohashes starting with '9xj'
    ./stip image split test -p Sentinel-2 -g 9xj -r -l 6
#### IMAGE COALESCE
In certain situations we require correlation of two image sets. This is useful for operations which require processing of image sets from diverse platforms at a particular spatiotemporal scope. We introduce the 'image coalesce' operation to provide the aforementioned functionality. Precisely, it splits a source image set so images exist at the same spatiotemporal scopes as a query image set.

    # split MODIS data to correlate with Sentinel-2 data with pixel
    #   coverage greater than 95% and cloud coverage less than 20%
    ./stip image coalesce test MODIS -p Sentinel-2 -x 0.95 -c 0.1
#### IMAGE FILL
Typically image datasets partition data into many tiles. The inherit tile bounds mean that often a single geohash spans multiple tiles. Therefore, when loading data, one image contains partial data whereas another contains the remaining data. The 'image fill' command attempts to identify image sets where 'complete' images may be built by combining multiple source images. This command launches a task on each cluster node to process data local to that machine. This command employs many of the same filtering criteria as 'image search' and 'image list' commands, enabling fine image processing filtering criteria.

    # attempt to fill all images in album test2 for the NAIP dataset
    ./stip image fill test2 -p NAIP

## TODO
- clean up documentation
- improve node logging
