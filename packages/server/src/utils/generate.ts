export function generateRequestUrl(
  protocol: string,
  host: string,
  port: number | null,
  originalUrl: string
): URL {
  return new URL(
    `${protocol}://${host}${port === null ? '' : `:${port}`}${originalUrl}`
  );
}

export function generateParserUrl(
  requestUrl: URL,
  remoteUrl: URL,
  href: string,
  engine?: string,
  redirect_url: string = 'get'
): string {
  const realURL = getRealURL(href, remoteUrl);

  const hash = realURL.hash; // save #hash
  realURL.hash = ''; // remove

  const urlParam = `?url=${encodeURIComponent(realURL.toString())}`;
  const engineParam = engine ? `&engine=${engine}` : '';

  return `${requestUrl.origin}/${redirect_url}${urlParam}${engineParam}${hash}`;
}

export function generateProxyUrl(
  requestUrl: URL,
  remoteUrl: URL,
  href: string,
  subProxy?: string
): string {
  const realHref = getRealURL(href, remoteUrl);

  const urlParam = `?url=${encodeURIComponent(realHref.href)}`;
  return `${requestUrl.origin}/proxy${subProxy ? `/${subProxy}` : ''}${urlParam}`;
}

function getRealURL(href: string, remoteUrl: URL) {
  return href.startsWith('http')
    ? new URL(href)
    : new URL(href, remoteUrl.href);
}
