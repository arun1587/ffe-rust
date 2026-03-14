#!/bin/bash
set -e

# Make sure osmium is installed
if ! command -v osmium &> /dev/null
then
    echo "osmium could not be found. Please install it (e.g., 'brew install osmium-tool' or 'sudo apt install osmium-tool')"
    exit 1
fi

echo "Downloading individual regions for Western France..."
curl -L -o bretagne-latest.osm.pbf "https://download.geofabrik.de/europe/france/bretagne-latest.osm.pbf"
curl -L -o basse-normandie-latest.osm.pbf "https://download.geofabrik.de/europe/france/basse-normandie-latest.osm.pbf"
curl -L -o haute-normandie-latest.osm.pbf "https://download.geofabrik.de/europe/france/haute-normandie-latest.osm.pbf"
curl -L -o pays-de-la-loire-latest.osm.pbf "https://download.geofabrik.de/europe/france/pays-de-la-loire-latest.osm.pbf"

echo "Merging regions into west-france.osm.pbf using osmium..."
osmium merge bretagne-latest.osm.pbf basse-normandie-latest.osm.pbf haute-normandie-latest.osm.pbf pays-de-la-loire-latest.osm.pbf -o west-france.osm.pbf --overwrite

echo "Cleaning up individual region files..."
rm bretagne-latest.osm.pbf basse-normandie-latest.osm.pbf haute-normandie-latest.osm.pbf pays-de-la-loire-latest.osm.pbf

echo "✅ Generation complete: west-france.osm.pbf"
