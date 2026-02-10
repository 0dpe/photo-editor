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

In a typical photography workflow, photos are first culled, then edited. Culling means to trim through all taken photos before editing to pick which ones are worth keeping and editing. Photo culling software often advertise high performance or AI-assisted culling. Culling usually isn't done in Adobe Lightroom, since it's slow and doesn't support advanced culling features. Instead, common choices include #link("https://home.camerabits.com/tour-photo-mechanic/")[Photo Mechanic], #link("https://narrative.so/select")[Narrative], and others. Culling software is often combined with editing software; popular culling or editing software include Adobe Lightroom, #link("https://www.captureone.com/en")[Capture One], #link("https://www.affinity.studio/photo-editing-software")[Affinity], #link("https://aftershoot.com/")[aftershoot], #link("https://skylum.com/luminar")[Luminar Neo], #link("https://www.evoto.ai/")[Evoto], #link("https://www.dxo.com/dxo-photolab/")[DxO PhotoLab], and others. These are desktop applications, all proprietary software with paid subscriptions. 

Web-based tools include mainly #link("https://next.polarr.com/")[Polarr Next] and #link("https://www.photopea.com/")[Photopea]. Photopea is laggy in my experience, while Polarr Next is fast. Polarr Next is a webapp that stores locally rather than on a cloud, and uses WebGPU (#link("https://next.polarr.com/#%3A~%3Atext%3DWhy%20is%20Polarr%20Next%20a%20web%20application%3F")[FAQ]) or WebGL2 (#link("https://github.com/Polarrco/polarr-next-sdk")[SDK]). Polarr Next is proprietary software with a paid subscription.

Open-source options include #link("https://rawtherapee.com/")[RawTherapee], #link("https://www.darktable.org/")[darktable], #link("https://www.digikam.org/")[digiKam], and others. #link("https://www.gimp.org/")[GIMP] and #link("https://krita.org/en/")[Krita] are similar to Photoshop, but do not natively support RAW images. 

= Proposal
Currently, there's no 

Todo: investigate multi-threaded CPU webapp, and loading files directly from camera on webapp.