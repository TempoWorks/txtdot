import { parseHTML } from 'linkedom';
import { Engine } from '../engine';

export class HandlerInput {
  private _data: string;
  private _url: string;
  private _window?: Window;

  constructor(data: string, url: string) {
    this._data = data;
    this._url = url;
  }

  get url(): string {
    return this._url;
  }

  get data(): string {
    return this._data;
  }

  get document(): Document {
    if (this._window) {
      return this._window.document;
    }

    this._window = parseHTML(this._data);
    return this._window.document;
  }
}

export interface HandlerOutput {
  content: string;
  textContent: string;
  title: string;
  lang: string;
}

export interface EngineOutput {
  content: string;
  textContent?: string;
  title?: string;
  lang?: string;
}

export const handlerSchema = {
  type: 'object',
  properties: {
    content: {
      type: 'string',
    },
    textContent: {
      type: 'string',
    },
    title: {
      type: 'string',
    },
    lang: {
      type: 'string',
    },
  },
};

export interface Engines {
  [key: string]: Engine;
}

export type EngineMatch<TParams extends RouteValues> = {
  pattern: string | string[];
  engine: EngineFunction<TParams>;
};

export interface RouteValues {
  [key: string]: string;
}

export type EngineFunction<TParams extends RouteValues> = (
  input: HandlerInput,
  ro: Route<TParams>
) => Promise<EngineOutput>;

export type MiddleFunction<TParams extends RouteValues> = (
  input: HandlerInput,
  ro: Route<TParams>,
  out: EngineOutput
) => Promise<EngineOutput>;

export type EnginesMatch<TParams extends RouteValues> = EngineMatch<TParams>[];

export interface Route<TParams extends RouteValues> {
  q: TParams;
  reverse: (req: { [K in keyof TParams]: string | number | boolean }) =>
    string | false;
}
