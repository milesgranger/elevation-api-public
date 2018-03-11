import os
from concurrent.futures import ProcessPoolExecutor, as_completed
from glob import glob
from fabric.api import local



def main():

    files = glob('./../ws-gap/data/geo/elevation/srtm-90/*.tif')

    futures = []

    with ProcessPoolExecutor(max_workers=8) as executor:
        for file in files:

            new_name = os.path.basename(file).replace('.tif', '.nc')
            command = f'gdal_translate -of netCDF -co "FORMAT=NC4" {file} ./data/{new_name}'

            future = executor.submit(local, command)
            futures.append(future)

    for future in as_completed(futures):
        print(future.result())


if __name__ == '__main__':
    main()
