import ipRangeCheck from 'ip-range-check';
import { isIP } from 'node:net';
import { LocalResourceError } from '../errors/main';
import dns from 'node:dns';

const blockedIPv4 = [
  '0.0.0.0/8',
  '10.0.0.0/8',
  '100.64.0.0/10',
  '127.0.0.0/8',
  '169.254.0.0/16',
  '172.16.0.0/12',
  '192.0.0.0/24',
  '192.0.2.0/24',
  '192.88.99.0/24',
  '192.168.0.0/16',
  '198.18.0.0/15',
  '198.51.100.0/24',
  '203.0.113.0/24',
  '224.0.0.0/4',
  '240.0.0.0/4',
];

const blockedIPv6Global = [
  '2001::/23',
  '2001:db8::/32',
  '2002::/16',
  '3ffe::/16',
  '3fff::/20',
];

export function isLocalResource(address: string): boolean {
  const family = isIP(address);

  if (family === 4) {
    return ipRangeCheck(address, blockedIPv4);
  }

  if (family === 6) {
    return (
      !ipRangeCheck(address, '2000::/3') ||
      ipRangeCheck(address, blockedIPv6Global)
    );
  }

  // Fail closed for invalid addresses
  return true;
}

export function checkHostname(hostname: string) {
  const host = hostname.replace(/^\[|\]$/g, '');

  if (isIP(host) && isLocalResource(host)) {
    throw new LocalResourceError();
  }
}

export function checkProtocol(protocol: string) {
  if (protocol !== 'http:' && protocol !== 'https:') {
    throw new Error('Unsupported URL protocol');
  }
}

export function safeLookup(
  hostname: string,
  options: dns.LookupOptions & { all?: boolean },
  callback: (...args: any[]) => void
) {
  const lookupOptions: dns.LookupAllOptions = {
    ...options,
    all: true,
  };

  dns.lookup(hostname, lookupOptions, (err, addresses) => {
    if (err) return callback(err);

    if (
      !addresses.length ||
      addresses.some(({ address }) => isLocalResource(address))
    ) {
      return callback(new LocalResourceError());
    }

    if (options.all) {
      return callback(null, addresses);
    }

    const { address, family } = addresses[0];
    callback(null, address, family);
  });
}
