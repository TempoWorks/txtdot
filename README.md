<h1 align="center">
  <a href="https://github.com/TxtDot/txtdot"><img src="https://github.com/TxtDot/.github/blob/main/imgs/TXTDot%20gh.png?raw=true" alt="txt." width="200"></a>
  <br>
  <a href="https://txtdot.github.io/documentation"><img alt="Documentation" src="https://img.shields.io/badge/Documentation-green"></a>
  <a href="https://github.com/TxtDot/instances"><img alt="Instances" src="https://img.shields.io/badge/Instances-green"></a>
  <br>
  <a href="https://github.com/TxtDot/txtdot/blob/main/LICENSE"><img alt="MIT license" src="https://img.shields.io/github/license/txtdot/txtdot?color=blue"></a>
  <a href="https://github.com/TxtDot/txtdot/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/TxtDot/txtdot?display_name=release"></a>
  <a href="https://matrix.to/#/#txtdot:matrix.org"><img alt="Matrix chat" src="https://img.shields.io/badge/chat-matrix-blue"></a>
</h1>

HTTP proxy that parses only text, links and pictures from pages
reducing internet traffic, removing ads and heavy scripts.
Mozilla's Readability library is used under the hood.

## Features

- Server-side page simplification
- Media proxy
- Image compression with Sharp
- Search with SearXNG
- Custom parsers for StackOverflow and SearXNG
- Handy API endpoints
- No client JavaScript
- Some kind of Material Design 3

## Running

### Development

```bash
npm install
npm run dev
```

### Production

```bash
npm install
npm run build
npm run start
```

### Docker

```bash
docker compose up -d
```

## Screenshots

<div align="center">
<img src="https://raw.githubusercontent.com/TxtDot/.github/main/imgs/ui_url_input.png" alt="Main page with URL input field">
<img src="https://raw.githubusercontent.com/TxtDot/.github/main/imgs/ui_search_page.png" alt="SearXNG search results page">
</div>

## Performance tests

txtdot is a great tool in case of slow internet connection or weak signal.
Here is the comparision of performance metrics from pagespeed.web.dev
between original page and proxied one.

"Mobile" test includes "Slow 4G" artificial network throttling.

<details>
<summary>Expand</summary>

|  |Original page|Proxied through txtdot|
|:-|:-----------:|:--------------------:|
|[Habr][habr-link] Desktop|![56%][habr-do-img]|![99%][habr-dt-img]|
|[Habr][habr-link] Mobile|![21%][habr-mo-img]|![100%][habr-mt-img]|
|[Medium][medium-link] Desktop|![44%][medium-do-img]|![100%][medium-dt-img]|
|[Medium][medium-link] Mobile|![36%][medium-mo-img]|![100%][medium-mt-img]|
|[Nginx Blog][nginx-link] Desktop|![53%][nginx-do-img]|![100%][nginx-dt-img]|
|[Nginx Blog][nginx-link] Mobile|![26%][nginx-mo-img]|![100%][nginx-mt-img]|

[habr-link]: https://habr.com/ru/articles/780692/
[habr-do-img]: https://raw.githubusercontent.com/TxtDot/.github/main/tests/habr/desktop_orig.png
[habr-dt-img]: https://raw.githubusercontent.com/TxtDot/.github/main/tests/habr/desktop_txtdot.png
[habr-mo-img]: https://raw.githubusercontent.com/TxtDot/.github/main/tests/habr/mobile_orig.png
[habr-mt-img]: https://raw.githubusercontent.com/TxtDot/.github/main/tests/habr/mobile_txtdot.png

[medium-link]: https://levelup.gitconnected.com/proxy-servers-how-proxies-work-0ec083fc1030
[medium-do-img]: https://raw.githubusercontent.com/TxtDot/.github/main/tests/medium/desktop_orig.png
[medium-dt-img]: https://raw.githubusercontent.com/TxtDot/.github/main/tests/medium/desktop_txtdot.png
[medium-mo-img]: https://raw.githubusercontent.com/TxtDot/.github/main/tests/medium/mobile_orig.png
[medium-mt-img]: https://raw.githubusercontent.com/TxtDot/.github/main/tests/medium/mobile_txtdot.png

[nginx-link]: https://www.nginx.com/blog/rate-limiting-nginx/
[nginx-do-img]: https://raw.githubusercontent.com/TxtDot/.github/main/tests/nginx-blog/desktop_orig.png
[nginx-dt-img]: https://raw.githubusercontent.com/TxtDot/.github/main/tests/nginx-blog/desktop_txtdot.png
[nginx-mo-img]: https://raw.githubusercontent.com/TxtDot/.github/main/tests/nginx-blog/mobile_orig.png
[nginx-mt-img]: https://raw.githubusercontent.com/TxtDot/.github/main/tests/nginx-blog/mobile_txtdot.png

</details>

## Credits

- [Readability.js](https://github.com/mozilla/readability)
- [🔗 LinkeDOM](https://github.com/WebReflection/linkedom)
- [Fastify web framework](https://github.com/fastify/fastify)
- [EJS](https://github.com/mde/ejs)
- [Axios](https://github.com/axios/axios)
- [DOMPurify](https://github.com/cure53/DOMPurify)
- [Sharp](https://github.com/lovell/sharp)
- [MicroMatch](https://github.com/micromatch/micromatch)
- [RouteParser](https://github.com/rcs/route-parser)
- [IconvLite](https://github.com/ashtuchkin/iconv-lite)
