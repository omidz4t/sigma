export const roles = [
  {
    "name": "Sigma",
    "role": "The hands at the terminal.",
    "description": "Your scripts and AI agents can send messages, files, and webxdc apps through a local CLI.",
    "href": "https://github.com/omidz4t/sigma",
    "image": "/assets/logo/moai-center.png"
  },
  {
    "name": "MADMAIL",
    "role": "A home for your messages.",
    "description": "A mail relay you can host yourself, handling delivery and storage for Delta Chat accounts.",
    "href": "https://github.com/themadorg/madmail",
    "image": "/assets/brands/madmail.png"
  },
  {
    "name": "Delta Chat",
    "role": "The conversation in your pocket.",
    "description": "The messenger where you read, reply, and open shared webxdc apps on your phone or computer.",
    "href": "https://delta.chat/"
  }
];
export const reasons = [
  {
    "title": "Your agent needs a way back to you.",
    "paragraphs": [
      "An agent can build something useful, but you still need to see it, try it, and respond. Sigma gives your scripts and AI agents a way to reach you in Delta Chat: send the result, receive your feedback, and keep working in the same conversation."
    ],
    "id": "return",
    "image": "/assets/friendship/return.webp"
  },
  {
    "title": "Preview a website without publishing it.",
    "paragraphs": [
      "When your agent builds a website, you often need a running preview server, an exposed port, or a tunnel to inspect it from your phone. For a static preview, the agent can instead package the HTML, CSS, JavaScript, and assets as a webxdc app and send it through Sigma.",
      "Open the app inside Delta Chat and explore it offline. The preview needs no public hosting or exposed website port. Server-side features still need adaptation: webxdc runs the packaged frontend, not your development server."
    ],
    "id": "preview",
    "image": "/assets/friendship/preview.webp"
  },
  {
    "title": "An interactive result, not just a screenshot.",
    "paragraphs": [
      "Ask your agent: “Find the latest news and show it to me in an interactive webxdc app.” The agent fetches the news and builds the view; you browse, filter, or choose what to investigate inside Delta Chat.",
      "With Sigma hooks enabled, app events can reach your agent, which can respond with new content or actions. The app must explicitly emit those events. Internet research happens on the agent’s side; the webxdc sandbox itself has no ordinary internet access."
    ],
    "id": "interactive",
    "image": "/assets/friendship/interactive.webp"
  },
  {
    "title": "Live interaction with Iroh.",
    "paragraphs": [
      "For live interaction, webxdc realtime channels use Iroh. Sigma can join the same shared app instance, listen for events, and exchange packets while your app is open. Direct peer-to-peer connections are used when possible, with encrypted relay transport available when needed.",
      "You do not need to publish a website or expose a preview port. Live interaction still needs connectivity. Realtime packets are temporary; use durable webxdc updates for state that should survive a disconnect."
    ],
    "id": "realtime",
    "image": "/assets/friendship/realtime.webp"
  },
  {
    "title": "A chat server on your own Wi-Fi.",
    "paragraphs": [
      "MADMAIL lets you host your own chat relay, including on a local network. With accounts and the required relay, discovery, and notification paths configured locally, your communication can stay within that network.",
      "That makes the combination useful beyond a public cloud setup. You control where the relay runs and how devices reach it; using the same Wi-Fi alone does not make public relay traffic local."
    ],
    "id": "local",
    "image": "/assets/friendship/local.webp"
  },
  {
    "title": "Encrypted, even through a public relay.",
    "paragraphs": [
      "You can also use community-run chatmail relays, including the public provisioning route used by a default Sigma setup. In the normal encrypted contact flow, messages and app updates are end-to-end encrypted; the relay transports ciphertext rather than reading your conversation.",
      "Relays store and forward messages so delivery can work when a device is offline. They remove messages after delivery or according to their retention policy, typically after a few weeks. Exact timing depends on the relay and device setup, rather than an instant-deletion guarantee."
    ],
    "id": "security",
    "image": "/assets/friendship/security.webp"
  },
  {
    "title": "A small server for real conversations.",
    "paragraphs": [
      "Delta Chat brings an established messaging ecosystem, while MADMAIL offers a lightweight, single-binary relay. You can handle conversations, files, and shared apps through the same messaging setup instead of hosting a separate preview service for every agent task.",
      "The compact deployment lets you size a server for your community’s traffic, attachments, and realtime connections."
    ],
    "id": "compact",
    "image": "/assets/friendship/compact.webp"
  },
  {
    "title": "Simple to start. Yours to run.",
    "paragraphs": [
      "MADMAIL provides an install --simple workflow for IP-based or domain-based deployments. Configure the relay for your network, connect your Delta Chat account, and let Sigma handle the agent side.",
      "Delta Chat is where you receive and explore the result. Sigma is how your agent reaches you and reacts. MADMAIL gives the conversation a home you can host. That is why this friendship lasts: the tools fit together around a practical feedback loop."
    ],
    "id": "together",
    "image": "/assets/friendship/together.webp"
  }
];
export const security = "Secure connections start with a QR code or private invitation. In an encrypted chat, your messages, files, and app updates are protected end to end. Delta Chat\u2019s encryption has undergone independent security audits. Keep your devices and invitation links safe, too.";
export const intro = "A private preview, a useful notification, or an app you can talk back to: the point is to bring your agent\u2019s work into the conversation you already have.";
export const narrationBlocks = [
  {
    "id": "title",
    "text": "Why this friendship lasts."
  },
  {
    "id": "lede",
    "text": "Your agent needs a way back to you."
  },
  {
    "id": "intro",
    "text": "A private preview, a useful notification, or an app you can talk back to: the point is to bring your agent’s work into the conversation you already have."
  },
  {
    "id": "role-0-name",
    "text": "Sigma"
  },
  {
    "id": "role-0-title",
    "text": "The hands at the terminal."
  },
  {
    "id": "role-0-body",
    "text": "Your scripts and AI agents can send messages, files, and webxdc apps through a local CLI."
  },
  {
    "id": "role-1-name",
    "text": "MADMAIL"
  },
  {
    "id": "role-1-title",
    "text": "A home for your messages."
  },
  {
    "id": "role-1-body",
    "text": "A mail relay you can host yourself, handling delivery and storage for Delta Chat accounts."
  },
  {
    "id": "role-2-name",
    "text": "Delta Chat"
  },
  {
    "id": "role-2-title",
    "text": "The conversation in your pocket."
  },
  {
    "id": "role-2-body",
    "text": "The messenger where you read, reply, and open shared webxdc apps on your phone or computer."
  },
  {
    "id": "delta-security",
    "text": "Secure connections start with a QR code or private invitation. In an encrypted chat, your messages, files, and app updates are protected end to end. Delta Chat’s encryption has undergone independent security audits. Keep your devices and invitation links safe, too."
  },
  {
    "id": "return-title",
    "text": "Your agent needs a way back to you."
  },
  {
    "id": "return-p0",
    "text": "An agent can build something useful, but you still need to see it, try it, and respond. Sigma gives your scripts and AI agents a way to reach you in Delta Chat: send the result, receive your feedback, and keep working in the same conversation."
  },
  {
    "id": "preview-title",
    "text": "Preview a website without publishing it."
  },
  {
    "id": "preview-p0",
    "text": "When your agent builds a website, you often need a running preview server, an exposed port, or a tunnel to inspect it from your phone. For a static preview, the agent can instead package the HTML, CSS, JavaScript, and assets as a webxdc app and send it through Sigma."
  },
  {
    "id": "preview-p1",
    "text": "Open the app inside Delta Chat and explore it offline. The preview needs no public hosting or exposed website port. Server-side features still need adaptation: webxdc runs the packaged frontend, not your development server."
  },
  {
    "id": "interactive-title",
    "text": "An interactive result, not just a screenshot."
  },
  {
    "id": "interactive-p0",
    "text": "Ask your agent: “Find the latest news and show it to me in an interactive webxdc app.” The agent fetches the news and builds the view; you browse, filter, or choose what to investigate inside Delta Chat."
  },
  {
    "id": "interactive-p1",
    "text": "With Sigma hooks enabled, app events can reach your agent, which can respond with new content or actions. The app must explicitly emit those events. Internet research happens on the agent’s side; the webxdc sandbox itself has no ordinary internet access."
  },
  {
    "id": "realtime-title",
    "text": "Live interaction with Iroh."
  },
  {
    "id": "realtime-p0",
    "text": "For live interaction, webxdc realtime channels use Iroh. Sigma can join the same shared app instance, listen for events, and exchange packets while your app is open. Direct peer-to-peer connections are used when possible, with encrypted relay transport available when needed."
  },
  {
    "id": "realtime-p1",
    "text": "You do not need to publish a website or expose a preview port. Live interaction still needs connectivity. Realtime packets are temporary; use durable webxdc updates for state that should survive a disconnect."
  },
  {
    "id": "local-title",
    "text": "A chat server on your own Wi-Fi."
  },
  {
    "id": "local-p0",
    "text": "MADMAIL lets you host your own chat relay, including on a local network. With accounts and the required relay, discovery, and notification paths configured locally, your communication can stay within that network."
  },
  {
    "id": "local-p1",
    "text": "That makes the combination useful beyond a public cloud setup. You control where the relay runs and how devices reach it; using the same Wi-Fi alone does not make public relay traffic local."
  },
  {
    "id": "security-title",
    "text": "Encrypted, even through a public relay."
  },
  {
    "id": "security-p0",
    "text": "You can also use community-run chatmail relays, including the public provisioning route used by a default Sigma setup. In the normal encrypted contact flow, messages and app updates are end-to-end encrypted; the relay transports ciphertext rather than reading your conversation."
  },
  {
    "id": "security-p1",
    "text": "Relays store and forward messages so delivery can work when a device is offline. They remove messages after delivery or according to their retention policy, typically after a few weeks. Exact timing depends on the relay and device setup, rather than an instant-deletion guarantee."
  },
  {
    "id": "compact-title",
    "text": "A small server for real conversations."
  },
  {
    "id": "compact-p0",
    "text": "Delta Chat brings an established messaging ecosystem, while MADMAIL offers a lightweight, single-binary relay. You can handle conversations, files, and shared apps through the same messaging setup instead of hosting a separate preview service for every agent task."
  },
  {
    "id": "compact-p1",
    "text": "The compact deployment lets you size a server for your community’s traffic, attachments, and realtime connections."
  },
  {
    "id": "together-title",
    "text": "Simple to start. Yours to run."
  },
  {
    "id": "together-p0",
    "text": "MADMAIL provides an install --simple workflow for IP-based or domain-based deployments. Configure the relay for your network, connect your Delta Chat account, and let Sigma handle the agent side."
  },
  {
    "id": "together-p1",
    "text": "Delta Chat is where you receive and explore the result. Sigma is how your agent reaches you and reacts. MADMAIL gives the conversation a home you can host. That is why this friendship lasts: the tools fit together around a practical feedback loop."
  }
];
