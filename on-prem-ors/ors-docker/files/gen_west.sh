#!/bin/bash

set -e

# 1. Create a directory for .poly files
mkdir -p polyfiles
cd polyfiles

# 2. Download .poly files
echo "Downloading .poly files..."
curl -s -o Bretagne.poly "https://polygons.openstreetmap.fr/get_poly.py?id=102740&params=0"
curl -s -o Normandie.poly "https://polygons.openstreetmap.fr/get_poly.py?id=3793170&params=0"
curl -s -o PaysDeLaLoire.poly "https://polygons.openstreetmap.fr/get_poly.py?id=8650&params=0"

# 3. Merge poly files into west.poly
echo "Merging .poly files into west.poly..."
{
  echo "west"
  for region in Bretagne Normandie PaysDeLaLoire; do
    sed '1d;$d' "${region}.poly"
  done
  echo "END"
} > west.poly

cd ..

# 4. Run osmium extract
echo "Extracting western France from france-latest.osm.pbf..."
osmium extract -p polyfiles/west.poly france-latest.osm.pbf -o west-france.osm.pbf

echo "✅ Extraction complete: west-france.osm.pbf"

