import { FastifyInstance } from 'fastify';

import { distributor } from '../../plugin_manager';
import { indexSchema } from '../../types/requests/browser';

import config from '../../config';

export default async function configurationRoute(fastify: FastifyInstance) {
  fastify.get(
    '/configuration',
    { schema: indexSchema },
    async (request, reply) => {
      return reply.view('/templates/configuration.ejs', {
        engines: distributor.engines_fallback,
        middlewares: distributor.middles_fallback,
        config,
        base_url: `(${request.protocol}) - ${request.hostname}`,
      });
    }
  );

  fastify.get(
    '/configuration/json',
    { schema: indexSchema },
    async (request, reply) => {
      reply.header('content-type', 'application/json');

      return {
        protocol: request.protocol,
        hostname: request.hostname,
        version: config.package.version,
        ...{ ...config.env, host: undefined, port: undefined },
        engines: distributor.engines_fallback.map((engine) => {
          return {
            ...engine,
            routes: engine.routes.map((route) => {
              return (route.route as any).spec;
            }),
          };
        }),
        middlewares: distributor.middles_fallback.map((middleware) => {
          return {
            ...middleware,
            routes: middleware.middles.map((middle) => {
              return (middle.route as any).spec;
            }),
            middles: undefined,
          };
        }),
        routes: [...config.dyn.routes].map((route) => {
          return route;
        }),
      };
    }
  );

  fastify.get(
    '/configuration/badges/big',
    { schema: indexSchema },
    async (request, reply) => {
      reply.header('content-type', 'image/svg+xml');
      return reply.view('/templates/big_badge.ejs', {
        config,
        base_url: `(${request.protocol}) - ${request.hostname}`,
      });
    }
  );
}
