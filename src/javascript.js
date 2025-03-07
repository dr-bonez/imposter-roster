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
 * @type {RTCConfiguration}
 */
const rtcConfig = {
  iceServers: [
    {
      urls: [
        'stun:stun.3wayint.com:3478',
        'stun:stun.f.haeder.net:3478',
        'stun:stun.siplogin.de:3478',
        'stun:stun.lovense.com:3478',
        'stun:stun.yesdates.com:3478',
        'stun:stun.sonetel.com:3478',
        'stun:stun.voipia.net:3478',
        'stun:stun.romaaeterna.nl:3478',
        'stun:stun.sipnet.com:3478',
        'stun:stun.siptrunk.com:3478',
        'stun:stun.antisip.com:3478',
        'stun:stun.signalwire.com:3478',
        'stun:stun.finsterwalder.com:3478',
        'stun:stun.ipfire.org:3478',
        'stun:stun.voipgate.com:3478',
        'stun:stun.radiojar.com:3478',
        'stun:stun.lleida.net:3478',
        'stun:stun.kanojo.de:3478',
        'stun:stun.peeters.com:3478',
        'stun:stun.nanocosmos.de:3478',
        'stun:stun.acronis.com:3478',
        'stun:stun.bridesbay.com:3478',
        'stun:stun.meetwife.com:3478',
        'stun:stun.ttmath.org:3478',
        'stun:stun.ringostat.com:3478',
        'stun:stun.files.fm:3478',
        'stun:stun.atagverwarming.nl:3478',
        'stun:stun.poetamatusel.org:3478',
        'stun:stun.cope.es:3478',
        'stun:stun.ncic.com:3478',
        'stun:stun.sipnet.net:3478',
        'stun:stun.verbo.be:3478',
        'stun:stun.mixvoip.com:3478',
        'stun:stun.ukh.de:3478',
        'stun:stun.moonlight-stream.org:3478',
        'stun:stun.stochastix.de:3478',
        'stun:stun.3deluxe.de:3478',
        'stun:stun.peethultra.be:3478',
        'stun:stun.nextcloud.com:443',
        'stun:stun.thinkrosystem.com:3478',
        'stun:stun.avigora.fr:3478',
        'stun:stun.diallog.com:3478',
        'stun:stun.axialys.net:3478',
        'stun:stun.oncloud7.ch:3478',
        'stun:stun.ru-brides.com:3478',
        'stun:stun.sip.us:3478',
        'stun:stun.heeds.eu:3478',
        'stun:stun.sonetel.net:3478',
        'stun:stun.romancecompass.com:3478',
        'stun:stun.allflac.com:3478',
        'stun:stun.genymotion.com:3478',
        'stun:stun.hot-chilli.net:3478',
        'stun:stun.business-isp.nl:3478',
        'stun:stun.flashdance.cx:3478',
        'stun:stun.bethesda.net:3478',
        'stun:stun.bitburger.de:3478',
        'stun:stun.frozenmountain.com:3478',
        'stun:stun.graftlab.com:3478',
        'stun:stun.jowisoftware.de:3478',
        'stun:stun.threema.ch:3478',
        'stun:stun.kaseya.com:3478',
        'stun:stun.fitauto.ru:3478',
        'stun:stun.vavadating.com:3478',
        'stun:stun.annatel.net:3478',
        'stun:stun.pure-ip.com:3478',
        'stun:stun.myspeciality.com:3478',
        'stun:stun.zepter.ru:3478',
        'stun:stun.zentauron.de:3478',
        'stun:stun.streamnow.ch:3478',
        'stun:stun.voip.blackberry.com:3478',
        'stun:stun.geesthacht.de:3478',
        'stun:stun.healthtap.com:3478',
        'stun:stun.dcalling.de:3478',
        'stun:stun.m-online.net:3478',
        'stun:stun.piratenbrandenburg.de:3478',
        'stun:stun.sipnet.ru:3478',
        'stun:stun.uabrides.com:3478',
        'stun:stun.nextcloud.com:3478',
        'stun:stun.baltmannsweiler.de:3478',
        'stun:stun.freeswitch.org:3478',
        'stun:stun.engineeredarts.co.uk:3478',
        'stun:stun.linuxtrent.it:3478',
        'stun:stun.imp.ch:3478',
        'stun:stun.telnyx.com:3478',
        'stun:stun.godatenow.com:3478',
        'stun:stun.skydrone.aero:3478',
        'stun:stun.alpirsbacher.de:3478',
        'stun:stun.1cbit.ru:3478',
        'stun:stun.root-1.de:3478',
        'stun:stun.technosens.fr:3478',
      ],
    },
  ],
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
        eventLog.innerHTML += `<p><b>The other player has connected.</b></p>`
        connected = true
        callButton.removeAttribute('disabled')
        break
      }
      case 'disconnected': {
        eventLog.innerHTML += `<p><b>The other player has disconnected.</b></p>`
        endCall().catch((e) => {
          console.error(e)
          callState = 'oncall'
        })
        connected = false
        callButton.setAttribute('disabled', true)
        break
      }
      case 'correct': {
        eventLog.innerHTML += `<p class="theirs"><b class="title">The other player correctly guessed your character in ${event.tries} tries!</b></p>`
        break
      }
      case 'incorrect': {
        eventLog.innerHTML += `<p class="theirs"><b class="title">The other player incorrectly guessed your character.</b></p>`
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
                callState = null
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
                  if (rejectOffer) rejectOffer()
                  else callState = null
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
                  if (rejectOffer) rejectOffer()
                  else callState = null
                }),
            )
            break
          }
          case 'reject': {
            if (rejectOffer) rejectOffer()
            else callState = null
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
              '<p class="mine"><b class="title">You guessed correctly!</b></p>'
          } else {
            document.getElementById(id).classList.remove('blackout')
            document.getElementById(id).classList.add('incorrect')
            eventLog.innerHTML +=
              '<p class="mine"><b class="title">You guessed incorrectly.</b></p>'
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
function call() {
  if (connected) {
    if (callState === null) {
      startCall().catch((e) => {
        console.error(e)
        callState = null
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

  setPeerConnection(new RTCPeerConnection(rtcConfig))

  for (const track of localAudioStream.getTracks()) {
    console.error(track)
    peerConnection.addTrack(track, localAudioStream)
  }
  if (offer) {
    peerConnection.setRemoteDescription(new RTCSessionDescription(offer))
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
    peerConnection.ontrack = (event) => {
      if (event.streams && event.streams[0]) {
        remoteAudio.srcObject = event.streams[0]
        resolve()
      }
    }
  })

  callState = 'oncall'
  callButton.innerHTML = 'Hang Up'
  callButton.removeAttribute('disabled')
}

async function endCall() {
  if (peerConnection) peerConnection.close()
  unsetPeerConnection()
  callState = null
  callButton.innerHTML = 'Call'
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
