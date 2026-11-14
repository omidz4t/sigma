# OpenCode → Sigma → Delta Chat

These are the container and hook scripts used in the website's real AI demo. They run OpenCode 2.2.7 with `opencode/glm-5.3-flash-free`; provider availability may change.

1. Build Sigma with `cargo build --release`, copy `target/release/sigma` next to this Dockerfile, then `docker build -t sigma-ai-showcase:local .`. Remove the copied binary after building.
2. Create an isolated writable home owned by UID 1001, with `work` and `recordings` directories. Mount it at `/home/demo`. Run the image with `--user 1001:1001 --cap-drop=ALL --security-opt no-new-privileges:true --read-only --tmpfs /tmp --pids-limit 256 --memory 4g --cpus 2`. Do not mount host credentials or the Docker socket; no port publishing is needed.
3. Inside the container, install `npm install -g @opencode/cli@2.2.7`, then `npx --yes skills add omidz4t/sigma --skill sigma --agent opencode --yes`.
4. Put your own Delta Chat invitation in a private `work/invite.txt`. Ask `opencode run --standalone --auto --model opencode/glm-5.3-flash-free` to build an offline HTML/CSS/JS app with `manifest.toml`, zip it as `.xdc`, use the Sigma skill to initialize and connect, then send it. Ask it to read the invitation without printing it, and save the returned chat/contact/message IDs in `work/result.json`. `init` creates the bot; `create-contact` connects it to you.
5. Copy both Python scripts to `work`, make `feedback-hook.py` executable, then run `python3 feedback-loop.py revise`. From that connected chat send `/no Use a light theme and add a live launch countdown.`. Run `python3 feedback-loop.py approve`, then send `/ok`.

The hook checks the connected chat ID, accepts only `/no` and `/ok`, and queues one event per message. The runner releases the account before asking OpenCode to act; this avoids concurrent account access. These commands are this demo's hooks, not built-in Sigma commands. The prompts deliberately restrict sending to the already connected alias `me`.

Record terminal sessions with `stty cols 80 rows 24; asciinema rec -c "python3 feedback-loop.py revise" /home/demo/recordings/revise.cast`. Review recordings before sharing: they can contain account addresses or other private output. The published demo redacts its bot address and used a separate, subsequently reset invitation.

`--auto` gives the agent permission to run tools inside this isolated container. It still has outbound network access and access to its mounted demo home; the container is not a network-free sandbox.
