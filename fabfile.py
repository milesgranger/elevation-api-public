import os
import ast
import zlib
import gzip
import tempfile
import logging
from concurrent.futures import ThreadPoolExecutor
from glob import glob
from fabric.api import local

logger = logging.getLogger(__name__)


def split_geotiff_file(source_file: str, destination_dir: str):

    # Use gdalinfo to get the coordinates represented in this file
    stdout = local(f'gdalinfo {source_file}', capture=True)

    upper_left_coords  = stdout.split('Upper Left  (')[-1].split(')')[0]
    upper_left_coords  = ast.literal_eval(f'({upper_left_coords})')
    lower_right_coords = stdout.split('Lower Right (')[-1].split(')')[0]
    lower_right_coords = ast.literal_eval(f'({lower_right_coords})')
    print(f'Upper left coords: {upper_left_coords}\nLower right coords: {lower_right_coords}')

    # Iterate over coordinates splitting the subwindow
    step_size = 5
    name = os.path.basename(source_file)
    for ulx in range(int(upper_left_coords[0]), int(lower_right_coords[0]), step_size):
        lrx = ulx + step_size
        for uly in range(int(lower_right_coords[1]), int(upper_left_coords[1]), step_size):
            lry = uly - step_size
            new_file_destination = os.path.join(destination_dir, f'{ulx}_{uly}_{lrx}_{lry}_{name}')
            local(f'gdal_translate -projwin {ulx} {uly} {lrx} {lry} {source_file} {new_file_destination}')


def convert_tif_to_netcdf(file_path: str, destination_dir: str):
    new_name = os.path.basename(file_path).lower().replace('.tif', '.nc')
    command = f'gdal_translate -of netCDF -co "FORMAT=NC4" {file_path} {os.path.join(destination_dir, new_name)}'
    local(command)


def compress(file_path: str, dest_dir):
    dest = f'{os.path.join(dest_dir, os.path.basename(file_path))}.gz'
    with open(file_path, 'rb') as rb, gzip.open(dest, 'wb') as wb:
        wb.write(rb.read())
    print(f'gzip {file_path} -> {dest}')

def process():
    """
    Process TIF files into a bunch of smaller NetCDF files with an associated
    summary.json file which lists what coordinates each title consists of.
    """

    raw_tifs = glob('./raw_download_tiffs/*.tif')
    print(f'Found {len(raw_tifs)} raw TIF files to process...')

    with tempfile.TemporaryDirectory() as tmp_dir:

        with ThreadPoolExecutor(max_workers=10) as executor:

            # Create new sub files from each .tif file
            _ = [i for i in executor.map(split_geotiff_file, raw_tifs, (tmp_dir for _ in range(len(raw_tifs))))]

            # Convert each tif to netCDF format.
            processed_tifs = glob(os.path.join(tmp_dir, '*'))
            _ = [i for i in executor.map(convert_tif_to_netcdf, processed_tifs, (tmp_dir for _ in range(len(processed_tifs))))]

            # Compress
            destination_dir = os.path.join(os.path.dirname(__file__), 'processed_netcdf_files')
            uncompressed_netcdf = glob(os.path.join(tmp_dir, '*.nc'))
            _ = [i for i in executor.map(compress, uncompressed_netcdf, (destination_dir for _ in range(len(processed_tifs))))]