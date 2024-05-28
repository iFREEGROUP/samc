<script setup>
import { ref, onMounted } from 'vue';
import { adjustImageToCanvas, debounce } from '../utils'
import TabView from 'primevue/tabview';
import TabPanel from 'primevue/tabpanel';
import Konva from 'konva'
import Dock from 'primevue/dock';
import Button from 'primevue/button';
import magicBlack from '../assets/cc-magic-black.svg?raw'
import magicWhite from '../assets/cc-magic-white.svg?raw'
import resetBlack from '../assets/reset-black.svg?raw'
import saveIcon from '../assets/save.svg?raw'
import { list_files, sam_from_base64, save_mask } from '../model';
import { watch } from 'vue';
import { useToast } from 'primevue/usetoast';
import Toast from 'primevue/toast';
import Slider from 'primevue/slider';


const toast = useToast();
const resetRef = ref(null)
const samcCanvas = ref(null);
const currentImage = ref({
    index: -1,
    url: "",
    path: "",
    mask_url: "",
    mask_path: "",
    rotate:0,
})

const configKonva = ref({
    width: 200,
    height: 200,
})

const configImage = ref({
    x: 0,
    y: 0,
    image: null,
    width: 100,
    height: 100,
})
const angle = ref(0)
const imageAngle = ref(0)
const annotationStatus = ref(false)
const showMask = ref(false)
const maskRef = ref(null)
const imageRef = ref(null)
const groupRef = ref(null)
const currentLowMask = ref(undefined)
const configMask = ref({
    x: 0,
    y: 0,
    image: null,
    width: 100,
    height: 100,
    name: 'mask',
    id: "mask"
})
const configGroup = ref({
    x: 0,
    y: 0,
    listening: false,
    offset: {
        offsetX: 0,
        offsetY: 0
    }
})
const circleItems = ref([])
const files = ref([])
const points = ref([])

const fetch_files = async () => {
    let { data } = await list_files();
    files.value = data.data
    if (!currentImage.value.url) {
        currentImage.value.index = 0
        currentImage.value.url = `${base_api}/${files.value[0].image_path}`
        currentImage.value.path = files.value[0].image_path
        currentImage.value.mask_path = `${files.value[0].mask_path}`
        currentImage.value.mask_url = `${base_api}/${files.value[0].mask_path}`
        currentImage.value.rotate = files.value[0].rotate
        imageAngle.value = files.value[0].rotate
    }
}

const base_api = `${import.meta.env.VITE_APP_API}/data`

const checkCurrentImageHandle = (file, index) => {
    currentImage.value.index = index
    currentImage.value.url = `${base_api}/${file.image_path}`
    currentImage.value.path = file.image_path
    currentImage.value.mask_path = `${file.mask_path}`
    currentImage.value.mask_url = `${base_api}/${file.mask_path}`
    currentImage.value.rotate = file.rotate
    console.log(file)
    imageAngle.value = file.rotate
    points.value = []
    circleItems.value = []
    

    let imageObj = new window.Image()
    imageObj.src = `${base_api}/${file.image_path}`
    imageObj.crossOrigin = 'Anonymous';
    imageObj.onload = () => {
        let size = adjustImageToCanvas(imageObj.width, imageObj.height, samcCanvas.value.offsetWidth, samcCanvas.value.offsetHeight)
        configImage.value.image = imageObj
        configImage.value.width = size.width
        configImage.value.height = size.height
    }
    if (file.mask_path) {
        let maskObj = new window.Image()
        maskObj.src = `${base_api}/${file.mask_path}`
        maskObj.crossOrigin = 'Anonymous';
        maskObj.onload = () => {
            let size = adjustImageToCanvas(maskObj.width, maskObj.height, samcCanvas.value.offsetWidth, samcCanvas.value.offsetHeight)
            configMask.value.image = maskObj
            configMask.value.width = size.width
            configMask.value.height = size.height
        }
        maskRef.value.getNode().show()
    } else {
        maskRef.value.getNode().hide()
    }
    showMask.value = !!file.mask_path

}

const resetHandle = e => {
    points.value = []
    circleItems.value = []
}

const onPointerDown = (e) => {
    // 左键为正提示，右键为负提示
    let isLeft = e.evt.button === 0
    var pos = e.target.getStage().getPointerPosition()
    var transform = e.target.getStage().getAbsoluteTransform().copy();
    transform.invert();
    var gPos = transform.point(pos);
    circleItems.value.push({
        x: gPos.x,
        y: gPos.y,
        radius: 5,
        fill: isLeft ? 'green' : 'red',
        stroke: 'black',
        strokeWidth: 2
    })
    points.value.push([parseInt(gPos.x), parseInt(gPos.y), isLeft ? 1 : 0])

    debounce(() => {
        imageRef.value.getNode().toDataURL({
            mimeType: 'image/png',
            quality: 1,
            callback: (dataUrl) => {
                if (!dataUrl) {
                    alert("图片加载失败，可能图片过大")
                    return
                }

                sam_from_base64(dataUrl, points.value,currentLowMask.value).then(({ data }) => {
                    let b64 = `data:image/png;base64,${data.data.mask}`
                    currentLowMask.value = data.data.low_mask
                    let maskObj = new window.Image()
                    maskObj.src = b64
                    maskObj.crossOrigin = 'Anonymous';
                    maskObj.onload = () => {
                        let size = adjustImageToCanvas(maskObj.width, maskObj.height, samcCanvas.value.offsetWidth, samcCanvas.value.offsetHeight)
                        configMask.value.image = maskObj
                        configMask.value.width = size.width
                        configMask.value.height = size.height
                    }
                    maskRef.value.getNode().show()
                })
            }
        })

    })()

}

const saveHandle = () => {
    save_mask(configMask.value.image.src, currentImage.value.path,imageAngle.value).then((r) => {
        console.log(r.data.data)
        files.value[currentImage.value.index].mask_path = r.data.data.file_name
        toast.add({ severity: 'success', summary: '保存成功', detail: '保存成功', life: 3000 });
    })
}

const setAnnotationStatus = (status) => {
    annotationStatus.value = !status
}
const slideendAngleHandle = (a) => {
    imageAngle.value = a.value
    let node = groupRef.value.getNode()
    node.rotation(0)
}
const changeAngleHandle = (a) => {
    let node = groupRef.value.getNode()
    node.x(configImage.value.width / 2)
    node.y(configImage.value.height / 2)
    node.offsetX(configImage.value.width / 2)
    node.offsetY(configImage.value.width / 2)
    node.rotation(a)

    console.log(node)
}

onMounted(async () => {
    groupRef.value.getNode().listening(annotationStatus.value)
    if (annotationStatus.value) {
        samcCanvas.value.style.cursor = 'crosshair'
    } else {
        samcCanvas.value.style.cursor = 'default'
    }
    configKonva.value.width = samcCanvas.value.offsetWidth
    configKonva.value.height = samcCanvas.value.offsetHeight
    await fetch_files()
    document.querySelector("#container").addEventListener('contextmenu', e => e.preventDefault())
    // console.log(currentImage.value)
    let imageObj = new window.Image()
    imageObj.src = currentImage.value.url
    imageObj.crossOrigin = 'Anonymous';
    imageObj.onload = () => {
        let size = adjustImageToCanvas(imageObj.width, imageObj.height, samcCanvas.value.offsetWidth, samcCanvas.value.offsetHeight)
        configImage.value.image = imageObj
        configImage.value.width = size.width
        configImage.value.height = size.height
    }
    if (currentImage.value.mask_path) {
        let maskObj = new window.Image()
        maskObj.src = currentImage.value.mask_url
        maskObj.crossOrigin = 'Anonymous';
        maskObj.onload = () => {
            let size = adjustImageToCanvas(maskObj.width, maskObj.height, samcCanvas.value.offsetWidth, samcCanvas.value.offsetHeight)
            configMask.value.image = maskObj
            configMask.value.width = size.width
            configMask.value.height = size.height
        }
        maskRef.value.getNode().show()
    } else {
        maskRef.value.getNode().hide()
    }
    showMask.value = !!currentImage.value.mask_path
    let node = groupRef.value.getNode()
    node.x(configImage.value.width / 2)
    node.y(configImage.value.height / 2)
    node.offsetX(configImage.value.width / 2)
    node.offsetY(configImage.value.width / 2)

    watch(annotationStatus, (val) => {
        node.listening(val)
        if (val) {
            samcCanvas.value.style.cursor = 'crosshair'
        } else {
            samcCanvas.value.style.cursor = 'default'
        }
    })
})
</script>
<template>
    <div class="grid grid-cols-8  h-full overflow-x-hidden">
        <div class="col-span-1 h-full">
            <div class="w-full h-full bg-zinc-100 ">
                <ul class="h-screen overflow-x-hidden overflow-y-scroll scrollbar-thin">
                    <li class="cursor-pointer m-2" v-for="(file, index) in files" :key="file.image_path"
                        @click="checkCurrentImageHandle(file, index)">
                        <div class="relative bg-white p-2 border rounded-md overflow-hidden hover:border-zinc-500"
                            :class="{ 'border-zinc-500': index == currentImage.index }">
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
            <div class="relative ">
                <div class="border-r border-zinc-100 mr-2">
                    <ul class="flex  items-center  p-2 bg-zinc-100">
                        <li class="mr-2">
                            <Button :severity="annotationStatus ? 'default' : 'secondary'"
                                @click="setAnnotationStatus(annotationStatus)">
                                <template #icon>
                                    <span v-show="!annotationStatus" v-html="magicBlack"></span>
                                    <span v-show="annotationStatus" v-html="magicWhite"></span>
                                </template>
                            </Button>
                        </li>
                        <li class="mr-2">
                            <Button :severity="'secondary'" @click="resetHandle" ref="resetRef">
                                <template #icon>
                                    <span v-html="resetBlack"></span>
                                </template>
                            </Button>
                        </li>
                        <li class="flex items-center mr-6">
                            <p class="w-12">{{ angle }}°</p>
                            <Slider v-model="angle" :max="180" :min="-180" @change="changeAngleHandle"
                                @slideend="slideendAngleHandle" class="w-64" />
                            <span class="ml-2 w-32">确认角度：{{ imageAngle }}</span>
                        </li>
                        <li class="mr-2">
                            <Button :severity="'secondary'" @click="saveHandle" icon="pi pi-save">
                                <template #icon>
                                    <span v-html="saveIcon"></span>
                                </template>
                            </Button>
                        </li>
                    </ul>
                </div>
                <div id="container" class="p-4">
                    <v-stage :config="configKonva">
                        <v-layer>
                            <v-group @pointerdown="onPointerDown" :config="configGroup" ref="groupRef">
                                <v-image :config="configImage" ref="imageRef"></v-image>
                                <v-image ref="maskRef" :config="configMask" :opacity="0.5"></v-image>
                                <v-circle v-for="item in circleItems" :key="item.id" :config="item"></v-circle>
                            </v-group>
                        </v-layer>
                    </v-stage>
                </div>

            </div>
        </div>
        
    </div>
</template>