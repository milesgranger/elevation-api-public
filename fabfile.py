import os
import sys
import ast
import zipfile
import gzip
import tempfile
import logging
from concurrent.futures import ThreadPoolExecutor, as_completed
from glob import glob
from fabric.api import local, task, hide

logging.basicConfig(level=logging.INFO)
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
    step_size = 10
    name = os.path.basename(source_file)
    for ulx in range(int(upper_left_coords[0]), int(lower_right_coords[0] - step_size), step_size):
        lrx = ulx + step_size
        for uly in range(int(lower_right_coords[1]), int(upper_left_coords[1] - step_size), step_size):
            lry = uly - step_size
            new_file_destination = os.path.join(destination_dir, f'{ulx}_{uly}_{lrx}_{lry}_{name}')
            local(f'gdal_translate -projwin {ulx} {uly} {lrx} {lry} {source_file} {new_file_destination}')


def convert_tif_to_netcdf(file_path: str, destination_dir: str):
    new_name = os.path.basename(file_path).replace('.tif', '.nc')
    command = f'gdal_translate -of netCDF -co "FORMAT=NC4" {file_path} {os.path.join(destination_dir, new_name)}'
    local(command)


def compress(file_path: str, dest_dir):
    dest = f'{os.path.join(dest_dir, os.path.basename(file_path))}.gz'
    with open(file_path, 'rb') as rb, gzip.open(dest, 'wb') as wb:
        wb.write(rb.read())
    print(f'gzip {file_path} -> {dest}')

@task
def process_tifs(source_dir: str):
    """
    Process TIF files into a bunch of smaller NetCDF files with an associated
    summary.json file which lists what coordinates each title consists of.
    """

    with tempfile.TemporaryDirectory() as tmp_dir:

        with ThreadPoolExecutor(max_workers=10) as executor:

            # Create new sub files from each .tif file
            """
            raw_tifs = glob(os.path.join('./raw_download_tiffs', '*.tif'))
            print(f'Found {len(raw_tifs)} raw TIF files to process...')
            futures = {executor.submit(split_geotiff_file, tif, tmp_dir): tif for tif in raw_tifs}
            for future in as_completed(futures):
                future.result()
            print('Done wtih dividing tifs.')
            """

            destination_dir = os.path.join(os.path.dirname(__file__), 'processed_netcdf_files')

            # Convert each tif to netCDF format.
            processed_tifs = glob(os.path.join(source_dir, '*.tif'))
            print(f'Found {len(processed_tifs)} to process!')
            futures = {executor.submit(convert_tif_to_netcdf, tif, destination_dir): tif for tif in processed_tifs}
            for future in as_completed(futures):
                future.result()

            # Compress
            # TODO: Add back compression once supported by NetCDF lib in Rust
            """
            uncompressed_netcdf = glob(os.path.join(tmp_dir, '*.nc'))
            futures = {executor.submit(compress, file, destination_dir): file for file in uncompressed_netcdf}
            for future in as_completed(futures):
                future.result()
            """

def run_command(command):
    with hide('output', 'running'):
        return local(command)

@task
def process_zip_hgt(source_dir: str, destination_dir: str):
    """
    Process a folder full of zipped hgt files, given source_dir and destination_dir arguments
    """

    zipped_files = glob(os.path.join(source_dir, '*.zip'))
    logger.info('Found {} files to process.'.format(len(zipped_files)))

    for i, file in enumerate(zipped_files):

        with tempfile.TemporaryDirectory() as tmp_dir:
            with zipfile.ZipFile(file, 'r') as f:
                f.extractall(tmp_dir)

            subfiles = glob(os.path.join(tmp_dir, '*/*'))

            # Might be one more zipped file inside this zip file
            with tempfile.TemporaryDirectory() as _tmp_dir:

                for subfile in filter(lambda f: f.lower().endswith('.zip'), subfiles):
                    with zipfile.ZipFile(subfile, 'r') as f:
                        f.extractall(_tmp_dir)
                subfiles.extend([f for f in glob(os.path.join(_tmp_dir, '*/*')) if os.path.isfile(f)])
                subfiles.extend([f for f in glob(os.path.join(_tmp_dir, '*')) if os.path.isfile(f)])
                backup = subfiles
                subfiles = [f for f in subfiles if f.lower().endswith('.hgt')]

                if not subfiles and len(backup) > 0:
                    raise RuntimeError('No subfiles in zipped file: "{}", original ones before filter: {}!'.format(file, backup))

                # Convert to NetCDF files
                futures = []
                with ThreadPoolExecutor(8) as executor:
                    for subfile in subfiles:
                        new_name = os.path.basename(subfile).lower().replace('.hgt', '.nc')
                        command = f'gdal_translate -of netCDF -co "FORMAT=NC4" {subfile} {os.path.join(destination_dir, new_name)}'
                        future = executor.submit(run_command, command)
                        futures.append(future)
                for future in as_completed(futures):
                    try:
                        future.result()
                    except Exception as e:
                        logger.critical('Failed converting to NetCDF: {}'.format(e))
                        raise e

        sys.stdout.write('\r{:.4f}% finished.'.format(((i+1) / len(zipped_files)) * 100))

