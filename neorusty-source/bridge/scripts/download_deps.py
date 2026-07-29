"""Download NeoForge 1.21.1 and Minecraft dependencies for the bridge."""
import json
import os
import sys
import urllib.request
import xml.etree.ElementTree as ET

NEOPATH = os.path.normpath(os.path.join(os.path.dirname(__file__), ".."))
LIBDIR = os.path.join(NEOPATH, "java", "lib")
NEOFORGE_VERSION = "21.1.1"
NEOFORGE_URL = f"https://maven.neoforged.net/releases/net/neoforged/neoforge/{NEOFORGE_VERSION}"
MAVEN_CENTRAL = "https://repo1.maven.org/maven2"
MOJANG_MAVEN = "https://libraries.minecraft.net"
MC_VERSION = "1.21.1"


def download(url, dest):
    print(f"  Downloading {os.path.basename(dest)}...")
    try:
        urllib.request.urlretrieve(url, dest)
        return True
    except Exception as e:
        print(f"  FAILED: {e}")
        return False


def download_neoforge():
    os.makedirs(LIBDIR, exist_ok=True)
    jar = os.path.join(LIBDIR, f"neoforge-{NEOFORGE_VERSION}-universal.jar")
    if os.path.exists(jar):
        print(f"  NeoForge jar already present ({os.path.getsize(jar)} bytes)")
        return True
    return download(f"{NEOFORGE_URL}/neoforge-{NEOFORGE_VERSION}-universal.jar", jar)


def get_minecraft_meta(version):
    """Fetch Minecraft version metadata JSON."""
    manifest_url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
    try:
        with urllib.request.urlopen(manifest_url) as resp:
            manifest = json.loads(resp.read())
    except Exception as e:
        print(f"  FAILED to fetch version manifest: {e}")
        return None
    for v in manifest.get("versions", []):
        if v["id"] == version:
            try:
                with urllib.request.urlopen(v["url"]) as resp:
                    return json.loads(resp.read())
            except Exception as e:
                print(f"  FAILED to fetch version meta: {e}")
                return None
    print(f"  Minecraft version {version} not found in manifest")
    return None


def download_minecraft_server(meta):
    if meta is None:
        return False
    server_info = meta.get("downloads", {}).get("server")
    if server_info:
        jar_path = os.path.join(LIBDIR, f"minecraft_server.{MC_VERSION}.jar")
        if os.path.exists(jar_path):
            print(f"  Minecraft server jar already present")
            return True
        return download(server_info["url"], jar_path)
    print("  No server download")
    return False


def download_minecraft_libs(meta):
    if meta is None:
        return 0
    libs = meta.get("libraries", [])
    print(f"  Found {len(libs)} library specs")
    downloaded = 0
    for lib in libs:
        if "downloads" not in lib:
            continue
        artifact = lib["downloads"].get("artifact")
        if artifact:
            url = artifact["url"]
            path = artifact.get("path", "")
            fname = os.path.basename(path) if path else os.path.basename(url)
            local = os.path.join(LIBDIR, fname)
            if os.path.exists(local):
                continue
            if download(url, local):
                downloaded += 1
            else:
                # Fallback: try maven central
                if "name" in lib:
                    parts = lib["name"].split(":")
                    if len(parts) >= 3:
                        g, a, v = parts[0], parts[1], parts[2]
                        url2, local2 = maven_coord_to_path(g, a, v)
                        if not os.path.exists(local2) and download(url2, local2):
                            downloaded += 1
        # Also handle classifiers (native libs etc.)
        for classifier_key, classifier_data in lib["downloads"].items():
            if classifier_key == "artifact":
                continue
            url = classifier_data["url"]
            fname = os.path.basename(url)
            local = os.path.join(LIBDIR, fname)
            if not os.path.exists(local):
                if download(url, local):
                    downloaded += 1
    return downloaded


def maven_coord_to_path(group, artifact, version, classifier=None):
    gpath = group.replace(".", "/")
    base = f"{artifact}-{version}"
    if classifier:
        base += f"-{classifier}"
    jar_name = f"{base}.jar"
    url = f"{MAVEN_CENTRAL}/{gpath}/{artifact}/{version}/{jar_name}"
    local = os.path.join(LIBDIR, jar_name)
    return url, local


def download_core_neoforge_libs(pom_path):
    """Download NeoForge's own declared POM dependencies."""
    ns = {"": "http://maven.apache.org/POM/4.0.0"}
    tree = ET.parse(pom_path)
    root = tree.getroot()
    # Get properties
    props = {}
    props_el = root.find("properties", ns)
    if props_el is not None:
        for child in props_el:
            tag = child.tag.split("}")[-1] if "}" in child.tag else child.tag
            props[tag] = child.text or ""
    deps = root.find("dependencies", ns)
    downloaded = 0
    if deps is not None:
        for dep in deps.findall("dependency", ns):
            g = dep.find("groupId", ns)
            a = dep.find("artifactId", ns)
            v = dep.find("version", ns)
            scope = dep.find("scope", ns)
            if g is not None and a is not None and v is not None:
                if scope is not None and scope.text in ("test", "provided"):
                    continue
                group, art, ver = g.text, a.text, v.text
                if ver.startswith("${"):
                    key = ver[2:-1]
                    ver = props.get(key, ver)
                classifier_el = dep.find("classifier", ns)
                classifier = classifier_el.text if classifier_el is not None else None
                dep_type_el = dep.find("type", ns)
                dep_type = dep_type_el.text if dep_type_el is not None else "jar"
                if dep_type != "jar":
                    continue
                url, local = maven_coord_to_path(group, art, ver, classifier)
                if not os.path.exists(local):
                    if download(url, local):
                        downloaded += 1
    return downloaded


def main():
    print("=== NeoForge Dependency Downloader ===\n")
    os.makedirs(LIBDIR, exist_ok=True)
    print(f"Target: {LIBDIR}")
    print(f"NeoForge version: {NEOFORGE_VERSION}")
    print(f"Minecraft version: {MC_VERSION}\n")

    print("[1/4] Downloading NeoForge universal jar...")
    download_neoforge()
    print()

    print("[2/4] Downloading NeoForge POM dependencies...")
    pom_path = os.path.join(LIBDIR, f"neoforge-{NEOFORGE_VERSION}.pom")
    if os.path.exists(pom_path):
        n = download_core_neoforge_libs(pom_path)
        if n:
            print(f"  Downloaded {n} NeoForge dependency jars")
    else:
        print("  POM not found, skipping")
    print()

    print("[3/4] Downloading Minecraft server...")
    meta = get_minecraft_meta(MC_VERSION)
    download_minecraft_server(meta)
    print()

    print("[4/4] Downloading Minecraft libraries...")
    if meta:
        n = download_minecraft_libs(meta)
        print(f"  Downloaded {n} new library files")
    print()

    print("=== Summary ===")
    total_size = 0
    count = 0
    for f in sorted(os.listdir(LIBDIR)):
        fpath = os.path.join(LIBDIR, f)
        if os.path.isfile(fpath):
            sz = os.path.getsize(fpath)
            total_size += sz
            count += 1
            print(f"  {f:55s} {sz // 1024:>6d} KB")
    print(f"\nTotal: {total_size // (1024*1024)} MB across {count} files")
    print("Done!")


if __name__ == "__main__":
    main()
