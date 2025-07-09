#!/usr/bin/env -S deno run --allow-env=GITHUB_TOKEN,GITHUB_REPOSITORY,GITHUB_OUTPUT --allow-write --allow-net=api.github.com
import { Octokit } from "https://esm.sh/octokit@5.0.3?dts";
const token = Deno.env.get("GITHUB_TOKEN");
if (!token) {
    console.error("GITHUB_TOKEN not set");
    Deno.exit(1);
}
const repoFull = Deno.env.get("GITHUB_REPOSITORY");
if (!repoFull) {
    console.error("GITHUB_REPOSITORY not set");
    Deno.exit(1);
}
const [owner, repo] = repoFull.split("/");
const octokit = new Octokit({ auth: token });

const { data: pulls } = await octokit.request(
    "GET /repos/{owner}/{repo}/pulls",
    { owner, repo, state: "open", per_page: 100 },
);

// newest‑updated PR with label "nightly"
const nightly = pulls
    .filter((pr) => pr.labels.some((l) => l.name === "nightly"))
    .sort((a, b) => Date.parse(b.updated_at) - Date.parse(a.updated_at))[0];

if (!nightly) {
    console.log("::warning::No open PR with label 'nightly' found.");
    Deno.exit(1);
}

console.log(
    `Found open PR #${nightly.number} with label 'nightly': ${nightly.html_url}`,
);

const outPath = Deno.env.get("GITHUB_OUTPUT");
if (outPath) {
    await Deno.writeTextFile(outPath, `ref=${nightly.head.sha}\n`, {
        append: true,
    });
}
Deno.exit(0);
