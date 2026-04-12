import * as core from "@actions/core";
import * as github from "@actions/github";

try {
    const commit_sha = github.context.sha;
    const branch = github.context.ref.replace("refs/heads/", "");
    core.info(`Building on branch: ${branch}`);

    const tags = [];
    let image_tag;
    // we are on a main branch
    if (branch === "main") {
        image_tag = `registry.gitlab.com/mwcaisse/application-images/arch-aur-builder:${commit_sha}`;
        tags.push(`registry.gitlab.com/mwcaisse/application-images/arch-aur-builder:latest`);
        tags.push(image_tag);

        //github images
        tags.push("ghcr.io/mwcaisse/aur-builder:latest");
        tags.push(`ghcr.io/mwcaisse/aur-builder:${commit_sha}`);

    } else {
        // when building for dev, we push to a different repository
        image_tag = `registry.gitlab.com/mwcaisse/application-images/arch-aur-builder-dev:${commit_sha}`;
        tags.push(image_tag);
        tags.push(`ghcr.io/mwcaisse/aur-builder-dev:${commit_sha}`);
    }

    // sanity check that image_tag was set, and it was put into the tags array
    if (image_tag.length === 0 || tags.indexOf(image_tag) === -1) {
        throw new Error("Unable to determine image tag or it was not pushed into tags array");
    }

    // Log out the tags that we are using
    core.info(`Tags: \n ${tags.map(tag => `\t${tag}`).join("\n")}`);
    core.info(`Image Tag: ${image_tag}`);

    // Set our outputs
    core.setOutput("tags", tags.join("\n"));
    core.setOutput("image_tag", image_tag);
    // TODO: For now version will just be the commit sha. Eventually we'll want an actual version here
    core.setOutput("version", commit_sha);
} catch (error) {
    core.setFailed(error.message);
}