import { createApp } from 'vue'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import PreviewPage from './components/PreviewPage.vue'
import './styles/main.css'

document.addEventListener('contextmenu', e => e.preventDefault())
document.documentElement.style.backgroundColor = 'transparent'
document.body.style.backgroundColor = 'transparent'

const app = createApp(PreviewPage)
app.use(ElementPlus)
app.mount('#app')