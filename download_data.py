import requests
import logging
import random
import bs4
from concurrent.futures import as_completed, ThreadPoolExecutor


logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


def download(i, link):
    filename = './30m_files/{}-{}'.format(i, link.split('/')[-1])
    r = requests.get(link, stream=True)
    with open(filename, 'wb') as f:
        for chunk in r.iter_content(chunk_size=1024):
            if chunk:
                f.write(chunk)
    return filename


if __name__ == '__main__':

    logger.info('Making initial request')
    resp = requests.get('http://viewfinderpanoramas.org/Coverage%20map%20viewfinderpanoramas_org3.htm')

    logger.info('Creating soup.')
    soup = bs4.BeautifulSoup(resp.content, 'lxml')

    logger.info('Begining to download files!')
    with ThreadPoolExecutor(5) as executor:
        jobs = {}
        for i, area in enumerate(soup.findAll('area')):
            l = area.get('href')
            jobs[executor.submit(download, i, l)] = l
        for future in as_completed(jobs):
            fname = jobs[future]
            try:
                file = future.result()
            except Exception as e:
                logger.warning('Could not download {} due to error: {}'.format(fname, e))
            else:
                logger.info('Downloaded file: {}'.format(file))
