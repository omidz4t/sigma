// Trusted, fixed bridge code. No agent-supplied JavaScript is evaluated.
async (iceServers, incoming = null) => {
  const ac = new AudioContext();
  await ac.resume();
  const peerOut = ac.createMediaStreamDestination();
  const providerOut = ac.createMediaStreamDestination();
  const recordingOut = ac.createMediaStreamDestination();
  const peer = new RTCPeerConnection({iceServers, bundlePolicy: 'max-bundle'});
  const provider = new RTCPeerConnection();
  // Delta clients normally keep a disabled camera track in the same stream
  // as audio. An empty video transceiver can leave their video-based player
  // waiting for its first frame. Supply black frames without camera access.
  const peerStream = new MediaStream(peerOut.stream.getAudioTracks());
  const compatibilityNodes = [];
  if (!incoming) {
    const canvas = document.createElement('canvas');
    canvas.width = canvas.height = 16;
    const paint = () => { const c = canvas.getContext('2d'); c.fillStyle = '#000'; c.fillRect(0, 0, 16, 16); };
    paint();
    const video = canvas.captureStream(1).getVideoTracks()[0];
    peerStream.addTrack(video);
    peer.addTrack(video, peerStream);
    // Keep frames coming even when the canvas contents do not change.
    const timer = setInterval(() => { paint(); video.requestFrame?.(); }, 1000);
    compatibilityNodes.push(canvas, video, timer);
  }
  peer.addTrack(peerOut.stream.getAudioTracks()[0], peerStream);
  provider.addTrack(providerOut.stream.getAudioTracks()[0], providerOut.stream);
  // Keep RTP flowing while the recipient is muted or before their first track.
  const clock = ac.createOscillator();
  const silence = ac.createGain(); silence.gain.value = 0;
  clock.connect(silence); silence.connect(peerOut); silence.connect(providerOut); silence.connect(recordingOut); clock.start();
  const state = {peer, provider, ac, nodes: [clock, silence, ...compatibilityNodes], error: null};
  state.stats = async () => {
    const leg = async pc => {
      const stats = [...(await pc.getStats()).values()];
      const sum = (type, key) => stats.filter(s => s.type === type && s.kind === 'audio').reduce((n, s) => n + (s[key] || 0), 0);
      const feedback = stats.filter(s => s.type === 'remote-inbound-rtp' && s.kind === 'audio');
      const sources = stats.filter(s => s.type === 'media-source' && s.kind === 'audio');
      return {connection: pc.connectionState, ice: pc.iceConnectionState,
        received: sum('inbound-rtp', 'packetsReceived'), sent: sum('outbound-rtp', 'packetsSent'),
        remoteReports: feedback.length,
        compatibilityFramesSent: stats.filter(s=>s.type==='outbound-rtp' && s.kind==='video').reduce((n,s)=>n+(s.framesEncoded||0),0),
        remotePacketsLost: feedback.length ? feedback.reduce((n,s)=>n+(s.packetsLost||0),0) : null,
        remoteRoundTripSeconds: feedback[0]?.roundTripTime ?? null,
        sourceEnergy: sources.length ? sources.reduce((n,s)=>n+(s.totalAudioEnergy||0),0) : null,
        audioDirections: pc.getTransceivers().filter(t=>t.receiver.track.kind==='audio').map(t=>t.currentDirection)};
    };
    return {peer: await leg(peer), provider: await leg(provider), audioContext: ac.state};
  };
  const relay = (event, destination) => {
    if (event.track.kind !== 'audio') return;
    try {
      const stream = new MediaStream([event.track]);
      // Keep the receiver's audio pipeline playing even though output is relayed.
      const sink = document.createElement('audio');
      sink.srcObject = stream; sink.autoplay = true; sink.volume = 0;
      document.body.appendChild(sink);
      sink.play().catch(e => state.error = String(e));
      state.nodes.push(sink);
      const source = ac.createMediaStreamSource(stream);
      source.connect(destination);
      const recordGain = ac.createGain(); recordGain.gain.value = 0.5;
      source.connect(recordGain); recordGain.connect(recordingOut);
      state.nodes.push(recordGain);
      state.nodes.push(source);
    } catch (e) { state.error = String(e); }
  };
  peer.ontrack = e => relay(e, providerOut);
  provider.ontrack = e => relay(e, peerOut);
  const ice = peer.createDataChannel('iceTrickling', {negotiated: true, id: 1});
  const mute = peer.createDataChannel('mutedState', {negotiated: true, id: 3});
  const pending = [];
  peer.onicecandidate = e => {
    const data = JSON.stringify(e.candidate);
    if (ice.readyState === 'open') ice.send(data); else pending.push(data);
  };
  ice.onopen = () => { for (const data of pending.splice(0)) ice.send(data); };
  ice.onmessage = e => {
    try { peer.addIceCandidate(JSON.parse(e.data)).catch(e => state.error = String(e)); }
    catch (e) { state.error = String(e); }
  };
  mute.onopen = () => mute.send(JSON.stringify({videoEnabled: false, audioEnabled: true}));
  // Delta control messages terminate here, never on the provider transport.
  window.sigmaCall = state;
  state.startRecording = () => {
    if (state.recorder) throw new Error('recording already started');
    const mimeType = 'audio/webm;codecs=opus';
    if (!MediaRecorder.isTypeSupported(mimeType)) throw new Error('Opus recording unavailable');
    state.recordChunks = []; state.recordBytes = 0; state.recordError = null;
    const recorder = new MediaRecorder(recordingOut.stream, {mimeType, audioBitsPerSecond: 64000});
    recorder.ondataavailable = e => {
      state.recordBytes += e.data.size;
      if (e.data.size > 262144 || state.recordBytes > 32 * 1024 * 1024) {
        state.recordError = 'recording size limit exceeded';
        if (recorder.state !== 'inactive') recorder.stop();
      } else if (e.data.size) state.recordChunks.push(e.data);
    };
    recorder.onerror = () => { state.recordError = 'media recorder failed'; };
    state.recorder = recorder; recorder.start(1000); return true;
  };
  state.stopRecording = async () => {
    if (state.recorder && state.recorder.state !== 'inactive') {
      await new Promise(resolve => { state.recorder.addEventListener('stop', resolve, {once:true}); state.recorder.stop(); });
    }
    return true;
  };
  state.recordChunk = async () => {
    if (state.recordError) throw new Error(state.recordError);
    const blob = state.recordChunks[0];
    if (!blob) return null;
    const bytes = new Uint8Array(await blob.arrayBuffer());
    const hex = Array.from(bytes, b => b.toString(16).padStart(2, '0')).join('');
    state.recordChunks.shift(); return hex;
  };
  const offer = async pc => {
    await pc.setLocalDescription(incoming && pc === peer ? await pc.createAnswer() : await pc.createOffer());
    if (pc.iceGatheringState !== 'complete') await new Promise(resolve => {
      const timer = setTimeout(done, 10000);
      function done() { clearTimeout(timer); pc.removeEventListener('icegatheringstatechange', changed); resolve(); }
      function changed() { if (pc.iceGatheringState === 'complete') done(); }
      pc.addEventListener('icegatheringstatechange', changed);
    });
    return pc.localDescription.sdp;
  };
  if (incoming) await peer.setRemoteDescription({type: 'offer', sdp: incoming});
  return {peer: await offer(peer), provider: await offer(provider)};
}
