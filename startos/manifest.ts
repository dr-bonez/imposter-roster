import { setupManifest } from '@start9labs/start-sdk'

export const manifest = setupManifest({
  id: 'imposter-roster',
  title: 'Imposter Roster',
  license: 'mit',
  wrapperRepo: 'https://github.com/dr-bonez/imposter-roster',
  upstreamRepo: 'https://github.com/dr-bonez/imposter-roster',
  supportSite: 'https://github.com/dr-bonez/imposter-roster',
  marketingSite: 'https://github.com/dr-bonez/imposter-roster',
  donationUrl: null,
  description: {
    short: 'A simple character guessing game with custom character packs',
    long: 'TODO',
  },
  assets: [],
  volumes: ['main'],
  images: {
    'imposter-roster': {
      source: {
        dockerBuild: {},
      },
    },
  },
  hardwareRequirements: {},
  alerts: {},
  dependencies: {},
})
