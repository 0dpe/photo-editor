#set page("us-letter")
#set text(size: 9pt)

#show link: underline

Unlike image formats like JPG or PNG, RAW images do not store rendered pixel color data; instead, they store electrical signals recorded by the camera sensor. 

#link("https://helpx.adobe.com/camera-raw/digital-negative.html")[DNG] is the universal RAW file format. Converting to DNG should not lose any information, other than perhaps some producer-specific metadata. DNG does not compress, though it seems to be on average around 8% smaller than a non-compressed CR3 file in my testing with 127 unedited files, CR3 converted to DNG through Adobe Lightroom. 

In a typical photography workflow, photos are first culled, then edited. Culling means to trim through all taken photos, before editing, to see which ones are worth keeping or editing. Photo culling software often advertise high performance or AI-assisted culling. Professionals usually don't cull in Lightroom, since it's slow and doesn't support many culling features. Instead, common choices include #link("https://home.camerabits.com/tour-photo-mechanic/")[Photo Mechanic], #link("https://aftershoot.com/")[aftershoot], #link("https://narrative.so/select")[Narrative], etc. These are desktop applications, all proprietary software and with paid subscriptions. As for online applications, there are mainly #link("https://next.polarr.com/")[Polarr Next] and #link("https://www.photopea.com/")[Photopea]. Photopea is laggy in my experience, while Polarr Next is fast. Polarr Next is a webapp that stores locally rather than on a cloud, and likely uses WebGPU (#link("https://next.polarr.com/#%3A~%3Atext%3DWhy%20is%20Polarr%20Next%20a%20web%20application%3F")[FAQ]) or WebGL2 (#link("https://github.com/Polarrco/polarr-next-sdk")[SDK]). Polarr Next is proprietary software with a paid subscription.  

Todo: investigate multi-threaded CPU webapp, and loading files directly from camera on webapp. 