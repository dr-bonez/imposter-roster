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
          } else {
            document.getElementById(id).classList.remove('blackout')
            document.getElementById(id).classList.add('incorrect')
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

const ws = new WebSocket('./ws')
ws.onmessage = (ev) => {
  const event = JSON.parse(ev.data)
  if (event.type === 'correct') {
    alert(
      `The other player correctly guessed your character in ${event.tries} tries!`,
    )
  } else if (event.type == 'incorrect') {
    alert('The other player incorrectly guessed your character')
  }
}
