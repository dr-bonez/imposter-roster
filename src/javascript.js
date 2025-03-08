guessing = false
function guess_mode() {
  guessing = !guessing
  console.log('guessing', guessing)
  if (guessing) {
    document.getElementById('game-board').classList.add('guessing')
    const btn = document.getElementById('guess-button')
    btn.classList.add('selected')
    btn.innerHTML = 'Stop Guessing'
  } else {
    document.getElementById('game-board').classList.remove('guessing')
    const btn = document.getElementById('guess-button')
    btn.classList.remove('selected')
    btn.innerHTML = 'Guess!'
  }
}

function new_game() {
  window.location.href = '/'
}

let hasPeerConnection = () => {}
/**
 * @type {RTCPeerConnection | undefined}
 */
let peerConnection
function getPeerConnection() {
  return new Promise((resolve) => {
    if (peerConnection) resolve(peerConnection)
    else hasPeerConnection = resolve
  })
}
function setPeerConnection(conn) {
  peerConnection = conn
  hasPeerConnection(peerConnection)
}
function unsetPeerConnection() {
  peerConnection = undefined
}

/**
 * @type {MediaStream | undefined}
 */
let localAudioStream
/**
 * @type {HTMLElement}
 */
let localAudio
/**
 * @type {HTMLElement}
 */
let remoteAudio
/**
 * @type {HTMLElement}
 */
let callButton
/**
 * @type {HTMLElement}
 */
let eventLog
/**
 * @type {(() => void) | undefined}
 */
let rejectOffer

let connected = false

/**
 *
 * @returns {Promise<RTCConfiguration>}
 */
async function rtcConfig() {
  const stunsRes = await fetch(
    'https://raw.githubusercontent.com/pradt2/always-online-stun/master/valid_hosts.txt',
  )
  const stuns = new TextDecoder()
    .decode(await stunsRes.arrayBuffer())
    .split('\n')
    .map((s) => s.trim())
    .filter((s) => !!s)
    .map((s) => `stun:${s}`)
  return {
    iceServers: [
      {
        urls: stuns,
      },
    ],
  }
}

/**
 * @type {WebSocket}
 */
let ws

function load() {
  eventLog = document.getElementById('event-log')
  localAudio = document.getElementById('local-audio')
  remoteAudio = document.getElementById('remote-audio')
  callButton = document.getElementById('call-button')

  ws = new WebSocket('./ws')
  ws.onmessage = (ev) => {
    const event = JSON.parse(ev.data)
    switch (event.type) {
      case 'connected': {
        eventLog.innerHTML += `<p class="theirs"><b class="title">The other player has connected.</b></p>`
        connected = true
        callButton.removeAttribute('disabled')
        break
      }
      case 'disconnected': {
        eventLog.innerHTML += `<p class="theirs"><b class="title">The other player has disconnected.</b></p>`
        endCall().catch((e) => console.error(e))
        connected = false
        callButton.setAttribute('disabled', true)
        break
      }
      case 'correct': {
        eventLog.innerHTML += `<p class="theirs"><b class="title">The other player <span style="color: green">correctly</span> guessed your character in ${event.tries} ${event.tries === 1 ? 'try' : 'tries'}!</b></p>`
        break
      }
      case 'incorrect': {
        eventLog.innerHTML += `<p class="theirs"><b class="title">The other player <span style="color: red">incorrectly</span> guessed your character.</b></p>`
        break
      }
      case 'message': {
        eventLog.innerHTML += `<p class="theirs"><b class="title">Them: </b>${event.content}</p>`
        break
      }
      case 'call': {
        switch (event.event.type) {
          case 'offer': {
            if (confirm('You are receiving a call! Accept?')) {
              startCall(event.event.offer).catch((e) => {
                console.error(e)
                return endCall().catch(console.error)
              })
            } else {
              ws.send(
                JSON.stringify({
                  type: 'call',
                  user_id,
                  event: { type: 'reject' },
                }),
              )
            }
            break
          }
          case 'answer': {
            getPeerConnection().then((p) =>
              p
                .setRemoteDescription(
                  new RTCSessionDescription(event.event.answer),
                )
                .catch((e) => {
                  console.error(e)
                  return endCall().catch(console.error)
                }),
            )
            break
          }
          case 'candidate': {
            getPeerConnection().then((p) =>
              p
                .addIceCandidate(new RTCIceCandidate(event.event.candidate))
                .catch((e) => {
                  console.error(e)
                  return endCall().catch(console.error)
                }),
            )
            break
          }
          case 'reject': {
            endCall(false).catch(console.error)
            break
          }
        }
        break
      }
    }
  }
}

function handle_click(id) {
  if (guessing) {
    let [row, col] = id.split('-')[1].split('_')
    console.log('guessing', row, col)
    fetch(`./guess?row=${row}&col=${col}`, {
      method: 'POST',
    }).then(async (res) => {
      if (res.status === 200) {
        const json = await res.json()
        if ('correct' in json) {
          if (json.correct) {
            document.getElementById(id).classList.remove('blackout')
            document.getElementById(id).classList.add('correct')
            eventLog.innerHTML +=
              '<p class="mine"><b class="title">You guessed <span style="color: green">correctly</span>!</b></p>'
          } else {
            document.getElementById(id).classList.remove('blackout')
            document.getElementById(id).classList.add('incorrect')
            eventLog.innerHTML +=
              '<p class="mine"><b class="title">You guessed <span style="color: red">incorrectly</span>.</b></p>'
          }
        } else {
          console.error('unexpected response', json)
        }
      } else {
        console.error(res)
      }
    })
  } else {
    document.getElementById(id).classList.toggle('blackout')
  }
}

let callState = null

/**
 *
 * @param {RTCSessionDescriptionInit} [offer]
 */
async function startCall(offer) {
  callState = 'calling'
  callButton.innerHTML = offer ? 'Connecting...' : 'Calling...'
  callButton.setAttribute('disabled', true)

  if (!localAudioStream) {
    let stream = await navigator.mediaDevices.getUserMedia({ audio: true })
    localAudioStream = stream
    localAudio.srcObject = stream
  }

  setPeerConnection(new RTCPeerConnection(await rtcConfig()))

  for (const track of localAudioStream.getTracks()) {
    peerConnection.addTrack(track, localAudioStream)
  }
  peerConnection.ontrack = (event) => {
    console.error(event)
    if (event.streams && event.streams[0]) {
      remoteAudio.srcObject = event.streams[0]
    }
  }
  if (offer) {
    await peerConnection.setRemoteDescription(new RTCSessionDescription(offer))
    const answer = await peerConnection.createAnswer()
    await peerConnection.setLocalDescription(answer)
    ws.send(
      JSON.stringify({
        type: 'call',
        user_id,
        event: { type: 'answer', answer: peerConnection.localDescription },
      }),
    )
  } else {
    offer = await peerConnection.createOffer()
    await peerConnection.setLocalDescription(offer)
    ws.send(
      JSON.stringify({
        type: 'call',
        user_id,
        event: { type: 'offer', offer: peerConnection.localDescription },
      }),
    )
  }
  peerConnection.onicecandidate = (ev) => {
    if (ev.candidate) {
      ws.send(
        JSON.stringify({
          type: 'call',
          user_id,
          event: { type: 'candidate', candidate: ev.candidate },
        }),
      )
    }
  }

  await new Promise((resolve, reject) => {
    rejectOffer = (e) => {
      reject(e)
      rejectOffer = undefined
    }
    peerConnection.onconnectionstatechange = (_) => {
      if (
        peerConnection.connectionState === 'failed' ||
        peerConnection.connectionState === 'closed'
      ) {
        reject('connection failed')
      } else if (peerConnection.connectionState === 'connected') {
        resolve()
      }
    }
  })

  callState = 'oncall'
  callButton.innerHTML = 'Hang Up'
  callButton.removeAttribute('disabled')
}

async function endCall(reject = true) {
  if (rejectOffer) rejectOffer()
  if (peerConnection) peerConnection.close()
  unsetPeerConnection()
  if (reject)
    ws.send(
      JSON.stringify({
        type: 'call',
        user_id,
        event: {
          type: 'reject',
        },
      }),
    )
  callState = null
  callButton.innerHTML = 'Call'
  if (connected) callButton.removeAttribute('disabled')
}

const user_id = document.cookie
  .split(';')
  .map((s) => s.trim())
  .find((s) => s.startsWith('user_id='))
  .split('=')[1]

function send_message() {
  const messagebar = document.getElementById('messagebar')
  const eventLog = document.getElementById('event-log')
  const message = messagebar.value
  ws.send(
    JSON.stringify({
      type: 'message',
      user_id,
      content: message,
    }),
  )
  eventLog.innerHTML += `<p class="mine"><b class="title">You: </b>${message}</p>`
  messagebar.value = ''
}

function call() {
  if (connected) {
    if (callState === null) {
      startCall().catch((e) => {
        console.error(e)
        endCall().catch(console.error(e))
      })
    } else if (callState === 'calling') {
      console.error('call already running')
    } else if (callState === 'oncall') {
      endCall().catch((e) => {
        console.error(e)
        callState = 'oncall'
      })
    }
  }
}
