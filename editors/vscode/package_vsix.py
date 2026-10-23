#!/usr/bin/env python3
"""Package the dependency-free local bridge without npm/vsce or network access."""

import argparse
import json
from pathlib import Path
import xml.etree.ElementTree as ET
from zipfile import ZIP_DEFLATED, ZipFile


def main():
    root = Path(__file__).resolve().parent
    package = json.loads((root / "package.json").read_text())
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=root / "dist" / f"{package['name']}-{package['version']}.vsix")
    output = parser.parse_args().output

    namespace = "http://schemas.microsoft.com/developer/vsx-schema/2011"
    ET.register_namespace("", namespace)
    manifest = ET.Element(f"{{{namespace}}}PackageManifest", Version="2.0.0")
    metadata = ET.SubElement(manifest, "Metadata")
    ET.SubElement(metadata, "Identity", Language="en-US", Id=package["name"], Version=package["version"], Publisher=package["publisher"])
    ET.SubElement(metadata, "DisplayName").text = package["displayName"]
    ET.SubElement(metadata, "Description").text = package["description"]
    ET.SubElement(metadata, "Categories").text = ",".join(package["categories"])
    properties = ET.SubElement(metadata, "Properties")
    for key, value in {
        "Microsoft.VisualStudio.Code.Engine": package["engines"]["vscode"],
        "Microsoft.VisualStudio.Code.ExtensionKind": ",".join(package["extensionKind"]),
        "Microsoft.VisualStudio.Code.ExecutesCode": "true",
    }.items():
        ET.SubElement(properties, "Property", Id=key, Value=value)
    installation = ET.SubElement(manifest, "Installation")
    ET.SubElement(installation, "InstallationTarget", Id="Microsoft.VisualStudio.Code")
    ET.SubElement(manifest, "Dependencies")
    assets = ET.SubElement(manifest, "Assets")
    for kind, filename in [
        ("Microsoft.VisualStudio.Code.Manifest", "package.json"),
        ("Microsoft.VisualStudio.Services.Content.Details", "README.md"),
        ("Microsoft.VisualStudio.Services.Content.License", "LICENSE"),
    ]:
        ET.SubElement(assets, "Asset", Type=kind, Path=f"extension/{filename}", Addressable="true")

    types_namespace = "http://schemas.openxmlformats.org/package/2006/content-types"
    types = ET.Element(f"{{{types_namespace}}}Types")
    for extension, content_type in {
        "json": "application/json",
        "cjs": "application/javascript",
        "md": "text/markdown",
        "vsixmanifest": "text/xml",
    }.items():
        ET.SubElement(types, f"{{{types_namespace}}}Default", Extension=extension, ContentType=content_type)
    ET.SubElement(types, f"{{{types_namespace}}}Override", PartName="/extension/LICENSE", ContentType="text/plain")

    output.parent.mkdir(parents=True, exist_ok=True)
    with ZipFile(output, "w", ZIP_DEFLATED) as archive:
        archive.writestr("extension.vsixmanifest", ET.tostring(manifest, encoding="utf-8", xml_declaration=True))
        archive.writestr("[Content_Types].xml", ET.tostring(types, encoding="utf-8", xml_declaration=True))
        for filename in ["package.json", "extension.cjs", "README.md", "LICENSE"]:
            archive.write(root / filename, f"extension/{filename}")
    print(output)


if __name__ == "__main__":
    main()
