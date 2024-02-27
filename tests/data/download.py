import json
import os
import shutil
import urllib.request

import rich.progress

with urllib.request.urlopen("http://obofoundry.org/registry/ontologies.jsonld") as response:
    obofoundry = json.load(response)

for ontology in obofoundry["ontologies"]:
    for product in ontology.get("products", []):
        print(product["ontology_purl"])

        product_id = os.path.basename(product["id"])
        if not product_id.endswith(".owl"):
            continue
        if os.path.exists(product_id):
            continue

        try:
            with urllib.request.urlopen(product["ontology_purl"]) as response:
                total = int(response.headers["Content-Length"])
                with rich.progress.wrap_file(response, total=total) as src:
                    with open(product_id, "wb") as dst:
                        shutil.copyfileobj(src, dst)
        except (TypeError, urllib.request.HTTPError, urllib.request.URLError):
            pass
