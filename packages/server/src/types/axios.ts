import origAxios, { CreateAxiosDefaults } from 'axios';
import { checkHostname, checkProtocol, safeLookup } from '../utils/blockLocal';
import http from 'node:http';
import https from 'node:https';

const httpAgent = new http.Agent({ lookup: safeLookup });
const httpsAgent = new https.Agent({ lookup: safeLookup });

const config: CreateAxiosDefaults<any> = {
  headers: {
    'User-Agent':
      'Mozilla/5.0 (X11; Linux x86_64; rv:159.0) Gecko/20100101 Firefox/159.0',
    Accept: 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
    'Sec-Fetch-Dest': 'document',
    'Sec-Fetch-Mode': 'navigate',
    'Sec-Fetch-Site': 'same-origin',
    'Sec-Fetch-User': '?1',
  },
  responseType: 'stream',
};

const axios = origAxios.create({
  ...config,
  proxy: false,

  httpAgent,
  httpsAgent,

  beforeRedirect(options) {
    checkProtocol(options.protocol);
    checkHostname(options.hostname);
  },
});

axios.interceptors.request.use((config) => {
  const url = new URL(config.url!, config.baseURL);

  checkProtocol(url.protocol);
  checkHostname(url.hostname);

  return config;
});

/**
 * Modified axios for blocking local resources
 */
export default axios;

/**
 *  Original axios
 */
export const oaxios = origAxios.create(config);
