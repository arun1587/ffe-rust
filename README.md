# ffe-rust

`ffe-rust` is a lightning-fast CLI scraper and router that discovers upcoming chess tournaments hosted by the French Chess Federation (FFE) and filters them based on geographic driving reachability using OpenRouteService (ORS).

## 🚀 Getting Started

### 1. Configuration (`.env`)
Create a `.env` file in the root of the project to tell `ffe-rust` how to utilize OpenRouteService:

```env
# Option A: Get a free API key at https://openrouteservice.org/dev/#/signup
ORS_API_KEY=your_key_here

# Option B: Use the heavily optimized Local on-prem Docker ORS instance
ORS_LOCAL_URL=http://localhost:8080/ors

# Note: If BOTH are provided, Hybrid mode is triggered (Local Routing + Remote Geocoding).
```

### 2. Running the App
Once configured, simply use `cargo run` and supply the exact city, department code, month, and maximum driving hours threshold:

```bash
cargo run -- --city Rennes --department 35 --month 4 --max-hours 2.0
```

* `reachable_events.json`: The fully filtered output list of reachable FFE tournaments.
* `all_events.json`: A raw dump of all parsed tournaments for the requested period.
* `geo_cache.json`: Caches API calls to dramatically speed up subsequent executions.

---

## 🗺 Local ORS Setup (On-Premise)

The public ORS API imposes heavily restricted free-tier rate limits (40 requests per minute). When scanning through ~100 chess tournaments, you will quickly get throttled. To solve this, you can run ORS locally!

Building the routing graph for all of France requires ~18GB+ of memory. To run this efficiently locally, you should download specific sub-regions.

### Prerequisites for Local ORS:
1. The **Docker** daemon must be running.
2. Install **osmium-tool** format processor:
   * Mac: `brew install osmium-tool`
   * Linux: `sudo apt install osmium-tool`

### Generating the Graph & Starting Docker:

1. **Navigate to the files directory:**
   ```bash
   cd on-prem-ors/ors-docker/files
   ```
2. **Download & Merge the OSM maps (Western France region by default):**
   ```bash
   # This will download Bretagne, Normandie, and Pays de la Loire via Geofabrik 
   # and merge them into a lightweight `west-france.osm.pbf` file automatically.
   ./gen_west.sh
   ```
3. **Start the ORS Docker container:**
   ```bash
   cd ../..
   docker compose up -d
   ```

*(Wait 1-3 minutes for the `driving-car` graph to compile locally. Check `docker logs ors-app -f` to monitor graph-building initialization. You will see an "ORS IS READY" status in the logs when it completes.)*
