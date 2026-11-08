import madurai from '../assets/polarities/madurai.png'
import vazarin from '../assets/polarities/vazarin.png'
import naramon from '../assets/polarities/naramon.png'
import zenurik from '../assets/polarities/zenurik.png'

export const polarities = {
  madurai: { name: 'Madurai', asset: madurai },
  vazarin: { name: 'Vazarin', asset: vazarin },
  naramon: { name: 'Naramon', asset: naramon },
  zenurik: { name: 'Zenurik', asset: zenurik },
} as const

export type PolarityKey = keyof typeof polarities
