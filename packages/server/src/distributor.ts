import axios, { oaxios } from './types/axios';
import micromatch from 'micromatch';
import DOMPurify from 'dompurify';
import { Readable } from 'stream';
import { NotHtmlMimetypeError } from './errors/main';
import { decodeStream, parseEncodingName } from './utils/http';
import replaceHref from './utils/replace-href';

import { Engine } from '@txtdot/sdk';
import { HandlerInput, HandlerOutput } from '@txtdot/sdk';
import config from './config';
import { parseHTML } from 'linkedom';
import { html2text } from './utils/html2text';

interface IEngineId {
  [key: string]: number;
}

export class Distributor {
  engines_id: IEngineId = {};
  fallback: Engine[] = [];
  list: string[] = [];
  constructor() {}

  engine(engine: Engine) {
    this.engines_id[engine.name] = this.list.length;
    this.fallback.push(engine);
    this.list.push(engine.name);
  }

  async handlePage(
    remoteUrl: string, // remote URL
    requestUrl: URL, // proxy URL
    engineName?: string,
    redirectPath: string = 'get'
  ): Promise<HandlerOutput> {
    const urlObj = new URL(remoteUrl);

    const webder_url = config.env.third_party.webder_url;

    const response = webder_url
      ? await oaxios.get(
          `${webder_url}/render?url=${encodeURIComponent(remoteUrl)}`
        )
      : await axios.get(remoteUrl);

    const data: Readable = response.data;
    const mime: string | undefined =
      response.headers['content-type']?.toString();

    if (mime && mime.indexOf('text/html') === -1) {
      throw new NotHtmlMimetypeError();
    }

    const engine = this.getFallbackEngine(urlObj.hostname, engineName);

    const output = await engine.handle(
      new HandlerInput(
        await decodeStream(data, parseEncodingName(mime)),
        remoteUrl
      )
    );

    const dom = parseHTML(output.content);

    // post-process
    // TODO: generate dom in handler and not parse here twice
    replaceHref(
      dom.document,
      requestUrl,
      new URL(remoteUrl),
      engineName,
      redirectPath
    );

    const purify = DOMPurify(dom);
    const content = purify.sanitize(dom.document.toString());
    const title = output.title || dom.document.title;
    const lang = output.lang || dom.document.documentElement.lang;
    const textContent =
      html2text(output, dom.document, title) ||
      'Text output cannot be generated.';

    return {
      content,
      textContent,
      title,
      lang,
    };
  }

  getFallbackEngine(host: string, specified?: string): Engine {
    if (specified) {
      return this.fallback[this.engines_id[specified]];
    }

    for (const engine of this.fallback) {
      if (micromatch.isMatch(host, engine.domains)) {
        return engine;
      }
    }

    return this.fallback[0];
  }
}
