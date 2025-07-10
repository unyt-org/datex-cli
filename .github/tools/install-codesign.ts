#!/usr/bin/env -S deno run --allow-env=GITHUB_TOKEN,GITHUB_OUTPUT --allow-write --allow-net=api.github.com,github.com,githubusercontent.com
import { Octokit } from "https://esm.sh/octokit@5.0.3?dts";

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
console.info(`Found release: ${release.data.tag_name} (${release.data.id})`);

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
const file = await Deno.open(artifactName, { create: true, write: true })
await response.body!.pipeTo(file.writable);
file.close();

console.info(`Downloaded ${artifactName} (${response.headers.get("content-length")} bytes)`);
const outPath = Deno.env.get("GITHUB_OUTPUT");
if (outPath) {
    await Deno.writeTextFile(outPath, `codesign=${Deno.cwd()}/${artifactName}\n`, {
        append: true,
    });
}
Deno.exit(0);
