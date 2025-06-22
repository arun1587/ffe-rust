# ffe-rust


# on prem ors

Public ORS API has rate limits, e.g., requests per minute/day.
On-prem lets you make unlimited calls

loading `france-latest.osm.pbf` which is around 5GiB has big memory/heap requirement
hence extract the regions from the base file i.e. `france-latest.osm.pbf` with the help
of `gen_west.sh` file. This script extacts regions like Bretage, Normandie and Paydelaloire.

```
cd on-prem-ors
docker compose up // this takes several minutes depending on loaded `.osm.pbf` file size.
```
