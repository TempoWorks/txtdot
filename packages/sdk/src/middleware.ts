import Route from 'route-parser';

import {
  HandlerInput,
  RouteValues,
  EngineOutput,
  MiddleFunction,
} from './types/handler';

interface IMiddle<TParams extends RouteValues> {
  route: Route;
  handler: MiddleFunction<TParams>;
}

export class Middleware {
  name: string;
  description: string;
  domains: string[];
  middles: IMiddle<any>[] = [];
  constructor(name: string, description: string, domains: string[] = []) {
    this.domains = domains;
    this.name = name;
    this.description = description;
  }

  route<TParams extends RouteValues>(
    path: string,
    handler: MiddleFunction<TParams>
  ) {
    this.middles.push({ route: new Route<TParams>(path), handler });
  }

  use<TParams extends RouteValues>(handler: MiddleFunction<TParams>) {
    this.middles.push({ route: new Route<{ path: string }>('*path'), handler });
  }

  async handle(input: HandlerInput, out: EngineOutput): Promise<EngineOutput> {
    const url = new URL(input.url);
    const path = url.pathname + url.search + url.hash;

    let processed_out = out;

    for (const middle of this.middles) {
      const match = middle.route.match(path);

      if (match) {
        processed_out = await middle.handler(
          input,
          {
            q: match,
            reverse: (req) => middle.route.reverse(req),
          },
          out
        );
      }
    }

    return processed_out;
  }
}
