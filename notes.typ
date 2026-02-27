#import "typst_assets/theme.typ": *

#set document(
  title: [Photo Editor],
  author: "Odpe",
  description: [Discovery Project],
  date: datetime(day: 8, month: 9, year: 2026),
)

#set page(
  paper: "us-letter",
  height: auto,
  fill: page-fill,
  margin: (x: 60pt, y: 60pt),
)

#set text(
  font: "Rethink Sans",
  size: 10pt,
  fill: text-main,
)

#set par(spacing: 1.3em)

#set enum(
  body-indent: 0.6em,
  indent: 1.0em,
)

#set list(
  body-indent: 0.6em,
  indent: 1.0em,
)

#set raw(theme: "/typst_assets/Cool Glow.tmTheme")


#show selector(title).or(heading): it => {
  set text(fill: text-heading)
  block(
    spacing: 1em,
    it,
  )
}

#show selector(title).or(heading.where(level: 1)): it => stack(
  spacing: 0.55em,
  it,
  line(length: 100%, stroke: 0.6pt + organization-line),
  v(0.2em),
)

#show link: set text(fill: text-link)
#show link: underline

#show raw: set text(
  font: "Roboto Mono",
  size: 1.1em,
)


#title()

= Research
Unlike image formats like JPG or PNG, RAW images do not store rendered pixel color data; instead, they store electrical signals recorded by the camera sensor. #link("https://helpx.adobe.com/camera-raw/digital-negative.html")[DNG] is the universal RAW file format. Converting to DNG should not lose any information, other than perhaps some producer-specific metadata. DNG does not compress, though it seems to be on average around 8% smaller than a non-compressed Canon CR3 file in my testing with 127 unedited files, CR3 converted to DNG through Adobe Lightroom.

In a typical photography workflow, photos are first culled, then edited. Culling means to trim through all taken photos before editing to pick which ones are worth keeping and editing. Photo culling software often advertise high performance or AI-assisted culling. Culling usually isn't done in Adobe Lightroom, since it's relatively slow and doesn't support advanced culling features. Instead, common choices include #link("https://home.camerabits.com/tour-photo-mechanic/")[Photo Mechanic], #link("https://narrative.so/select")[Narrative], and others. Culling software is often combined with editing software; popular culling or editing software include Adobe Lightroom, #link("https://www.captureone.com/en")[Capture One], #link("https://www.affinity.studio/photo-editing-software")[Affinity], #link("https://aftershoot.com/")[aftershoot], #link("https://skylum.com/luminar")[Luminar Neo], #link("https://www.evoto.ai/")[Evoto], #link("https://www.dxo.com/dxo-photolab/")[DxO PhotoLab], and others. These are desktop applications, all proprietary software with paid subscriptions. 

Web-based tools include mainly #link("https://next.polarr.com/")[Polarr Next] and #link("https://www.photopea.com/")[Photopea]. Polarr Next is a webapp that stores locally rather than on a cloud, and uses WebGPU (#link("https://next.polarr.com/#%3A~%3Atext%3DWhy%20is%20Polarr%20Next%20a%20web%20application%3F")[FAQ]) or WebGL2 (#link("https://github.com/Polarrco/polarr-next-sdk")[SDK]). Polarr Next is proprietary software with a paid subscription.

Open source options include #link("https://rawtherapee.com/")[RawTherapee], #link("https://www.darktable.org/")[darktable], #link("https://www.digikam.org/")[digiKam], and others; these are desktop applications. #link("https://www.gimp.org/")[GIMP], #link("https://krita.org/en/")[Krita], and #link("https://pixieditor.net/")[PixiEditor] are similar to Photoshop, but do not natively support RAW images. Open source web-based image editing tools include #link("https://graphite.art/")[Graphite], #link("https://github.com/igorski/bitmappery")[BitMappery], #link("https://github.com/viliusle/miniPaint")[miniPaint], and others, but they do not yet support RAW image formats. Open source webapps that support RAW editing include #link("https://github.com/CyberTimon/RapidRAW")[RapidRAW], #link("https://github.com/poesterlin/GiRAF")[GiRAF], and others; these are nascent projects. 

= Proposal
Currently, there's no major, established open source image culling or editing webapp that supports RAW images. 

Todo: investigate multi-threaded CPU webapp, and loading files directly from camera on webapp.