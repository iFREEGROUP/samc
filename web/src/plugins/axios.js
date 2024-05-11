import axios from "axios";

axios.defaults.baseURL = `${import.meta.env.VITE_APP_API || ''}/api`;
const _axios = axios.create({
    baseURL: `${import.meta.env.VITE_APP_API || ''}/api`,
    timeout: 1000000,
});
_axios.interceptors.request.use(
    function (config) {

        // config.headers.Authorization = 'Bearer ' +accessToken.value
        // console.log('Adding token to header', accessToken.value)
        return config;
    },
    function (error) {
        // Do something with request error
        return Promise.reject(error);
    }
);
// Add a response interceptor
_axios.interceptors.response.use(
    function (response) {
        // Do something with response data
        return response;
    },
    function (error) {

        return Promise.reject(error);
    }
);
export default _axios;