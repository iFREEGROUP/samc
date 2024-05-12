<script setup>
import { ref, onMounted } from 'vue';
import { adjustImageToCanvas } from '../utils'
import TabView from 'primevue/tabview';
import TabPanel from 'primevue/tabpanel';
import Konva from 'konva'
import Dock from 'primevue/dock';
import Button from 'primevue/button';
import magicBlack from '../assets/cc-magic-black.svg?raw'
import magicWhite from '../assets/cc-magic-white.svg?raw'
import resetBlack from '../assets/reset-black.svg?raw'
import { list_files, sam_from_base64 } from '../model';












const annaRef = ref(null)
const annaStatus = ref(false)

const resetRef = ref(null)
const samcCanvas = ref(null);

const currentImage = ref(null)

const initKonva = (width, height) => {
    var stage = new Konva.Stage({
        container: 'container',
        width: width,
        height: height,
    });
    var layer = new Konva.Layer();

    var group = new Konva.Group({
        x: 0,
        y: 0,
        rotation: 0,
        listening: false,
        draggable: true,
    });


    let imageObj = new window.Image()
    let maskObj = new window.Image()

    var points = [];

    imageObj.src = currentImage.value
    imageObj.crossOrigin = 'Anonymous';
    imageObj.onload = () => {
        let size = adjustImageToCanvas(imageObj.width, imageObj.height, width, height)


        var yoda = new Konva.Image({
            x: 0,
            y: 0,
            image: imageObj,
            width: size.width,
            height: size.height,
        });
        
        
        group.on('mousedown', function (e) {
            console.log(e)
            var circle = new Konva.Circle({
                x: e.evt.layerX,
                y: e.evt.layerY,
                radius: 7,
                fill: 'red',
                stroke: 'black',
                strokeWidth: 4
            });
            group.add(circle);
            points.push([e.evt.layerX, e.evt.layerY, false])
            console.log(yoda)
            if (annaStatus) {
                yoda.toDataURL({
                    mimeType: 'image/png',
                    quality: 1,
                    callback: function (dataUrl) {
                        if (!dataUrl) {
                            alert("图片加载失败，可能图片过大")
                            return
                        }
                        sam_from_base64(dataUrl, points).then(({ data }) => {
                            maskObj.src = `data:image/png;base64,${data.data.mask}`
                            maskObj.onload = () => {
                                var mask = new Konva.Image({
                                    x: 0,
                                    y: 0,
                                    image: maskObj,
                                    width: size.width,
                                    height: size.height,
                                    name: 'mask',
                                    id: "mask"
                                });
                                mask.opacity(0.5)
                                if (group.findOne("#mask")) {
                                    group.findOne("#mask").destroy()
                                }
                                group.add(mask)
                                mask.zIndex(1)
                            }
                        })
                    }
                })

            }
        });
        // add the shape to the layer
        group.add(yoda);

        resetRef.value.$el.addEventListener('click', e => {
            group.removeChildren()

            if (yoda) {
                group.add(yoda)

            }
            console.log(group)
            console.log(yoda)

        })

        annaRef.value.$el
            .addEventListener('click', e => {
                annaStatus.value = !annaStatus.value
                if (annaStatus.value) {
                    group.listening(true)
                    samcCanvas.value.style.cursor = 'crosshair'

                } else {
                    group.listening(false)
                    samcCanvas.value.style.cursor = 'default'
                }

            })
    }





    layer.add(group);
    stage.add(layer);
}

const files = ref([])

const fetch_files = () => {
    list_files().then(({ data }) => {
        files.value = data.data
        if (!currentImage.value) {
            currentImage.value = `${base_api}/${files.value[0]}`
            if (samcCanvas.value) {
                console.log("init konva")
                initKonva(samcCanvas.value.offsetWidth, samcCanvas.value.offsetHeight)
            }
        }
    })
}

const base_api = `${import.meta.env.VITE_APP_API}/data`


const checkCurrentImageHandle = (file) => {
    currentImage.value = `${base_api}/${file}`
    if (samcCanvas.value) {
        initKonva(samcCanvas.value.offsetWidth, samcCanvas.value.offsetHeight)
    }
}

onMounted(() => {

    fetch_files()
    document.querySelector("#container").addEventListener('contextmenu',e=>e.preventDefault())
})
</script>
<template>
    <div class="grid grid-cols-8  h-full">
        <div class="col-span-1 h-full">
            <div class="w-full h-full bg-zinc-100 ">
                <ul class="h-screen overflow-x-hidden overflow-y-scroll  scrollbar-thin">
                    <li class="cursor-pointer p-2" v-for="file in files" :key="file" @click="checkCurrentImageHandle(file)">
                        <div class="relative">
                            <Image :src="`${base_api}/${file}`" />
                            <span class="my-2">labal this is a long cat hahahahahah.jpg</span>
                        </div>
                    </li>
                </ul>
            </div>
        </div>
        <div class="col-span-6 py-5 relative bg-zinc-200" ref="samcCanvas">
            <div id="container"></div>
            <div class="absolute inset-y-0 left-0 border-r border-zinc-100 mr-2">
                <ul class="flex flex-col items-center justify-center h-full p-2 bg-zinc-100">
                    <li>
                        <Button :severity="annaStatus ? 'default' : 'secondary'" ref="annaRef">

                            <template #icon>
                                <span v-show="!annaStatus" v-html="magicBlack"></span>
                                <span v-show="annaStatus" v-html="magicWhite"></span>
                            </template>
                        </Button>
                    </li>
                    <li>
                        <Button :severity="'secondary'" ref="resetRef">

                            <template #icon>
                                <span v-html="resetBlack"></span>
                            </template>
                        </Button>
                    </li>
                </ul>
            </div>
        </div>
        <div class="col-span-1">
            <TabView>
                <TabPanel header="Header I">
                    <p class="m-0">
                        Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut
                        labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco
                        laboris nisi ut aliquip ex ea commodo
                        consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu
                        fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia
                        deserunt mollit anim id est laborum.
                    </p>
                </TabPanel>
                <TabPanel header="Header II">
                    <p class="m-0">
                        Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque
                        laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto
                        beatae vitae dicta sunt explicabo. Nemo enim
                        ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni
                        dolores eos qui ratione voluptatem sequi nesciunt. Consectetur, adipisci velit, sed quia non
                        numquam eius modi.
                    </p>
                </TabPanel>

            </TabView>
        </div>
    </div>
</template>