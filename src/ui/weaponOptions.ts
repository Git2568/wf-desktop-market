import type { RivenWeapon } from '../api/market'
import type { SearchOption } from '../components/SearchSelect.vue'

export type WeaponOption = SearchOption & { rivenType: string }

export const rivenTypeLabels: Record<string, string> = {
  rifle: '步枪', shotgun: '霰弹枪', pistol: '手枪', melee: '近战武器', zaw: 'ZAW', kitgun: 'Kitgun',
}

export function weaponTypeLabel(type: string) { return rivenTypeLabels[type] ?? type }

export function weaponOptionsFor(weapons: RivenWeapon[]): WeaponOption[] {
  const collator = new Intl.Collator('en', { sensitivity: 'base' })
  return weapons.map(item => ({
    value: item.slug,
    label: item.i18n['zh-hans']?.name ?? item.i18n.en?.name ?? item.slug,
    secondary: item.i18n.en?.name ?? item.slug,
    rivenType: item.riven_type,
  })).sort((a, b) => collator.compare(a.secondary ?? a.value, b.secondary ?? b.value))
}
