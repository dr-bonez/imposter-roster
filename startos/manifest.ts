import { setupManifest } from '@start9labs/start-sdk'

export const manifest = setupManifest({
  id: 'guess-who',
  title: 'Guess Who',
  license: 'mit',
  wrapperRepo: 'https://github.com/dr-bonez/guess-who',
  upstreamRepo: 'https://github.com/dr-bonez/guess-who',
  supportSite: 'https://github.com/dr-bonez/guess-who',
  marketingSite: 'https://github.com/dr-bonez/guess-who',
  donationUrl: null,
  description: {
    short: 'A simple "Guess Who" game with custom character packs',
    long: 'TODO',
  },
  assets: [],
  volumes: ['main'],
  images: {
    'guess-who': {
      source: {
        dockerBuild: {},
      },
    },
  },
  hardwareRequirements: {},
  alerts: {},
  dependencies: {},
})
