import presentation from '../data/riven-attribute-presentation.json'

type PresentationGroup = { key: string; label: string; attributes: string[] }
type PresentationFile = { version: number; groups: PresentationGroup[] }
type AttributeLike = { slug: string; name: string; english_name?: string }

const data = presentation as PresentationFile
const preferredOrder = ['core', 'element', 'physical', 'faction', 'other', 'spliced', 'fusion']
const groups = [...data.groups].sort((a, b) => {
  const ai = preferredOrder.indexOf(a.key)
  const bi = preferredOrder.indexOf(b.key)
  return (ai < 0 ? preferredOrder.length : ai) - (bi < 0 ? preferredOrder.length : bi)
})
const groupBySlug = new Map<string, { label: string; rank: number; index: number }>()
groups.forEach((group, index) => group.attributes.forEach((slug, rank) => groupBySlug.set(slug, { label: group.label, rank, index })))
const otherLabel = groups.find(group => group.key === 'other')?.label ?? '其他'

export function attributePresentation(attribute: AttributeLike) {
  const known = groupBySlug.get(attribute.slug)
  return { group: known?.label ?? otherLabel, groupIndex: known?.index ?? preferredOrder.indexOf('other'), rank: known?.rank ?? Number.POSITIVE_INFINITY }
}

export function sortAttributePresentation<T extends AttributeLike>(attributes: readonly T[]) {
  return [...attributes].sort((a, b) => {
    const pa = attributePresentation(a)
    const pb = attributePresentation(b)
    if (pa.groupIndex !== pb.groupIndex) return pa.groupIndex - pb.groupIndex
    if (pa.rank !== pb.rank) return pa.rank - pb.rank
    return (a.english_name ?? a.name).localeCompare(b.english_name ?? b.name, 'en', { sensitivity: 'base' })
  })
}
