import origAxios, { CreateAxiosDefaults } from 'axios';
import { isLocalResource, isLocalResourceURL } from '../utils/islocal';
import { LocalResourceError } from '../errors/main';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const config: CreateAxiosDefaults<any> = {
  headers: {
    'User-Agent':
      'Mozilla/5.0 (X11; Linux x86_64; rv:150.0) Gecko/20100101 Firefox/150.0',
    'Sec-Fetch-Dest': 'document',
    'Sec-Fetch-Mode': 'navigate',
    'Sec-Fetch-Site': 'same-origin',
    'Sec-Fetch-User': '?1',
  },
  responseType: 'stream',
};

const axios = origAxios.create(config);

axios.interceptors.response.use(
  (response) => {
    if (isLocalResource(response.request.socket.remoteAddress)) {
      throw new LocalResourceError();
    }

    return response;
  },
  async (error) => {
    if (await isLocalResourceURL(new URL(error.config?.url))) {
      throw new LocalResourceError();
    }

    throw error;
  }
);

/**
 * Modified axios for blocking local resources
 */
export default axios;

/**
 *  Original axios
 */
export const oaxios = origAxios.create(config);
