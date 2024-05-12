import axios from "../plugins/axios";


/**
 * 
 * @param {String} base64 
 * @param {any} inputs 
 */
export const sam_from_base64 = (base64,inputs) => {
    return axios.post(`/sam_base64`,{image:base64,inputs})
}

export const list_files = ()=> axios.get(`/files`)