import axios from "../plugins/axios";


/**
 * 
 * @param {String} base64 
 * @param {any} inputs 
 */
export const sam_from_base64 = (base64,inputs,masks = undefined) => {

    let payload = {
        image:base64,
        inputs:inputs
    }
    if(masks) payload.masks = masks

    return axios.post(`/sam_base64`,payload)
}

export const list_files = ()=> axios.get(`/files`)

export const save_mask = (mask,ori_image_name,rotate)=> axios.post(`/masks`,{mask,ori_image_name,rotate})