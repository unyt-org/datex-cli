#!/usr/bin/env -S deno run -A
import { Octokit } from "https://esm.sh/octokit@5.0.3?dts";
// --allow-env=GITHUB_TOKEN --allow-write --allow-net
const token = Deno.env.get("GITHUB_TOKEN");
if (!token) {
    console.error("GITHUB_TOKEN not set");
    Deno.exit(1);
}

const [owner, repo] = "indygreg/apple-platform-rs".split("/");
const octokit = new Octokit({ auth: token });

const getTag = async (version: string) => {
    if (version === "latest") {
        const latestRelease = await octokit.request("GET /repos/{owner}/{repo}/releases/latest", {
            owner,
            repo,
        });
        if (!latestRelease.data.tag_name)
            throw new Error("No tag name found in latest release");
        return latestRelease.data.tag_name;
    }
    return `apple-codesign/${version}`;
}
const getArtifactName = (version: string) => {
    const prefix = `apple-codesign-${version}-${Deno.build.arch}-`;
    const platform = Deno.build.os;
    if (platform === "darwin") {
        return `${prefix}apple-${platform}.tar.gz`;
    } else if (platform === "linux") {
        return `${prefix}unknown-linux-musl.tar.gz`;
    } else if (platform === "windows") {
        return `${prefix}pc-windows-msvc.zip`;
    } else {
        throw new Error(`Unsupported platform: ${platform}`);
    }
}

const version = Deno.args[0] || "latest";
const tag = await getTag(version);

const release = await octokit.request(
  "GET /repos/{owner}/{repo}/releases/tags/{tag}",
  {
    owner,
    repo,
    tag,
  }
);
console.info(`Found release: ${tag} (${release.data.id})`);

if (!release.data.assets || release.data.assets.length === 0) {
    console.error("No assets found in release");
    Deno.exit(1);
}
const artifactName = getArtifactName(tag.replace("apple-codesign/", ""));
const asset = release.data.assets.find(
  (a) => a.name === artifactName
);
if (!asset) {
    console.error(`No asset found with name ${artifactName}`);
    Deno.exit(1);
}
console.info(`Found asset: ${asset.name} (${asset.id})`);
const downloadUrl = asset.browser_download_url;
console.info(`Downloading from ${downloadUrl}`);
const response = await fetch(downloadUrl);
const tarFile = await Deno.open(artifactName, { create: true, write: true })
await response.body!.pipeTo(tarFile.writable);

const outDir = "./bin";
const isZip = artifactName.endsWith(".zip");
const archive = isZip ? "rcodesign.zip" : "rcodesign.tar.gz";

await Deno.mkdir(outDir, { recursive: true });
const extractCmd = isZip
  ? new Deno.Command("unzip", {args: ["-o", archive, "-d", outDir]})
  : new Deno.Command("tar", {args: ["-xzf", archive, "-C", outDir]});

await extractCmd.output();

const binPath = `${outDir}/rcodesign`;
await Deno.chmod(binPath, 0o755);

Deno.exit(0);