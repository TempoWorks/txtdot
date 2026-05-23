import Route from 'route-parser';

import {
  HandlerInput,
  EngineFunction,
  RouteValues,
  EngineOutput,
} from './types/handler';

import { NoHandlerFoundError } from './types/errors';

interface IRoute<TParams extends RouteValues> {
  route: Route;
  handler: EngineFunction<TParams>;
}

export class Engine {
  name: string;
  description: string;
  domains: string[];
  routes: IRoute<any>[] = [];
  constructor(name: string, description: string, domains: string[] = []) {
    this.domains = domains;
    this.name = name;
    this.description = description;
  }

  route<TParams extends RouteValues>(
    path: string,
    handler: EngineFunction<TParams>
  ) {
    this.routes.push({ route: new Route<TParams>(path), handler });
  }

  async handle(input: HandlerInput): Promise<EngineOutput> {
    const url = new URL(input.url);
    const path = url.pathname + url.search + url.hash;
    for (const route of this.routes) {
      const match = route.route.match(path);

      if (match) {
        return await route.handler(input, {
          q: match,
          reverse: (req) => route.route.reverse(req),
        });
      }
    }

    throw new NoHandlerFoundError(`${path}. [${this.name}]`);
  }
}
