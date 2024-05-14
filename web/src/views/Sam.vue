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
import { list_files, sam_from_base64, save_mask } from '../model';
import { watch } from 'vue';
import { useToast } from 'primevue/usetoast';
import Toast from 'primevue/toast';








function debounce(func, delay) {
    let timeoutId;

    return function () {
        const context = this;
        const args = arguments;

        clearTimeout(timeoutId);

        timeoutId = setTimeout(function () {
            func.apply(context, args);
        }, delay);
    };
}

const toast = useToast();
const annaRef = ref(null)
const annaStatus = ref(false)

const resetRef = ref(null)
const samcCanvas = ref(null);

const currentImage = ref({
    url: "",
    path: "",
    mask_url:""
})

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
    

    imageObj.src = currentImage.value.url
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


        group.on('pointerdown', function (e) {


            // console.log(e)
            var pos = group.getStage().getPointerPosition()
            var transform = group.getAbsoluteTransform().copy();
            transform.invert();
            var gPos = transform.point(pos);
            // console.log(pos,gPos)
            var circle = new Konva.Circle({
                x: gPos.x,
                y: gPos.y,
                radius: 7,
                fill: 'red',
                stroke: 'black',
                strokeWidth: 4
            });
            group.add(circle);
            points.push([parseInt(gPos.x), parseInt(gPos.y), false])
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
                console.log("annaStatus::", annaStatus.value)
                if (annaStatus.value) {
                    group.listening(true)
                    samcCanvas.value.style.cursor = 'crosshair'

                } else {
                    group.listening(false)
                    samcCanvas.value.style.cursor = 'default'
                }

            })

        document.addEventListener("keyup", function (event) {
            if (event.key === "n") {
                annaStatus.value = !annaStatus.value
                console.log("annaStatus::", annaStatus.value)
                if (annaStatus.value) {
                    group.listening(true)
                    samcCanvas.value.style.cursor = 'crosshair'

                } else {
                    group.listening(false)
                    samcCanvas.value.style.cursor = 'default'
                }
            }
            if (event.key === "Enter") {
                //保存图片，并让group状态回到初始状态
                save_mask(maskObj.src, currentImage.value.path).then((r) => {
                    toast.add({ severity: 'success', summary: '保存成功', detail: '保存成功', life: 3000 });
                })
            }
        })
    }
    if (currentImage.value.mask_url) {
        maskObj.src = currentImage.value.mask_url
        maskObj.crossOrigin = 'Anonymous';
        maskObj.onload = () => {
            let size = adjustImageToCanvas(imageObj.width, imageObj.height, width, height)

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
    }




    layer.add(group);
    stage.add(layer);
}

const files = ref([])

const fetch_files = () => {
    list_files().then(({ data }) => {
        files.value = data.data
        if (!currentImage.value.url) {
            currentImage.value.url = `${base_api}/${files.value[0].image_path}`
            currentImage.value.path = files.value[0].image_path
            currentImage.value.mask_url = `${base_api}/${files.value[0].mask_path}`
            if (samcCanvas.value) {

                initKonva(samcCanvas.value.offsetWidth, samcCanvas.value.offsetHeight)
            }
        }
    })
}

const base_api = `${import.meta.env.VITE_APP_API}/data`


const checkCurrentImageHandle = (file) => {
    currentImage.value.url = `${base_api}/${file}`
    currentImage.value.path = file
    if (samcCanvas.value) {
        initKonva(samcCanvas.value.offsetWidth, samcCanvas.value.offsetHeight)
    }
}

onMounted(() => {

    fetch_files()
    document.querySelector("#container").addEventListener('contextmenu', e => e.preventDefault())
})
</script>
<template>
    <div class="grid grid-cols-8  h-full overflow-x-hidden">
        <div class="col-span-1 h-full">
            <div class="w-full h-full bg-zinc-100 ">
                <ul class="h-screen overflow-x-hidden overflow-y-scroll scrollbar-thin">
                    <li class="cursor-pointer m-2" v-for="file in files" :key="file.image_path"
                        @click="checkCurrentImageHandle(file.image_path)">
                        <div class="relative bg-white p-2 border rounded-md overflow-hidden hover:border-zinc-500">
                            <!-- <Image :src="`${base_api}/${file}`" /> -->
                            <span class="my-2 text-wrap break-all" :class="{
                        'text-green-500': file.mask_path
                    }">{{ file.image_path }}</span>
                        </div>
                    </li>
                </ul>
            </div>
        </div>
        <div class="col-span-7 bg-zinc-200" ref="samcCanvas">
            <Toast />
            <div class="relative flex">
                <div class="border-r border-zinc-100 mr-2">
                    <ul class="flex flex-col items-center justify-center  p-2 bg-zinc-100">
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
                <div id="container" class="p-2"></div>
            </div>
        </div>
        <!-- <div class="col-span-1">
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
        </div> -->
    </div>
</template>