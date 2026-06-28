#!/bin/sh
set -eu

PORT="${PORT:-8080}"

BLOCKED_IPV4="
0.0.0.0/8
10.0.0.0/8
100.64.0.0/10
127.0.0.0/8
169.254.0.0/16
172.16.0.0/12
192.0.0.0/24
192.0.2.0/24
192.168.0.0/16
198.18.0.0/15
198.51.100.0/24
203.0.113.0/24
224.0.0.0/4
240.0.0.0/4
"

BLOCKED_IPV6="
::/128
::1/128
::ffff:0:0/96
fc00::/7
fe80::/10
ff00::/8
2001:db8::/32
"

iptables -w -F
iptables -w -X
iptables -w -P INPUT DROP
iptables -w -P FORWARD DROP
iptables -w -P OUTPUT DROP

iptables -w -A INPUT -i lo -j ACCEPT
iptables -w -A OUTPUT -o lo -j ACCEPT
iptables -w -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
iptables -w -A OUTPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
iptables -w -A INPUT -p tcp --dport "$PORT" -j ACCEPT

for cidr in $BLOCKED_IPV4; do
  iptables -w -A OUTPUT -d "$cidr" -j REJECT
done

iptables -w -A OUTPUT -p tcp -j ACCEPT

if command -v ip6tables >/dev/null 2>&1; then
  ip6tables -w -F
  ip6tables -w -X
  ip6tables -w -P INPUT DROP
  ip6tables -w -P FORWARD DROP
  ip6tables -w -P OUTPUT DROP

  ip6tables -w -A INPUT -i lo -j ACCEPT
  ip6tables -w -A OUTPUT -o lo -j ACCEPT
  ip6tables -w -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
  ip6tables -w -A OUTPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
  ip6tables -w -A INPUT -p tcp --dport "$PORT" -j ACCEPT

  for cidr in $BLOCKED_IPV6; do
    ip6tables -w -A OUTPUT -d "$cidr" -j REJECT
  done

  ip6tables -w -A OUTPUT -p tcp -j ACCEPT
fi

touch /tmp/firewall-ready
while :; do
  sleep 3600
done
