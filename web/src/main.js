import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import PrimeOne from 'primevue/themes/primeone';
import PrimeVueStyled from 'primevue/styled';
import { ShadcnPreset } from './theme/preset';
import VueKonva from 'vue-konva';

const app = createApp(App)

app.use(VueKonva);
app.use(PrimeVueStyled, {
    theme: {
        base: PrimeOne,
        preset: ShadcnPreset,
        options: {
            darkModeSelector: '.dark',
        }
    }
});


app.mount('#app')