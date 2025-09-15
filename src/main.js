import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import store from "./store";
import "./assets/css/global.css";

// 引入Vant
import {
  Button,
  Dialog,
  Icon,
  Cell,
  CellGroup,
  Tabbar,
  TabbarItem,
  NavBar,
  Field,
  Form,
  List,
  PullRefresh,
  Card,
  Tag,
  Image as VanImage,
  Col,
  Row,
  Grid,
  GridItem,
  Swipe,
  SwipeItem,
  Divider,
  Tab,
  Tabs,
  Skeleton,
  Badge,
} from "vant";
import "vant/lib/index.css"; // Vite 环境保留样式引入

const app = createApp(App);

// 注册Vant组件
app.use(Button);
app.use(Dialog);
// Vant 4 中 Toast 改为函数式调用，无需注册插件
app.use(Icon);
app.use(Cell);
app.use(CellGroup);
app.use(Tabbar);
app.use(TabbarItem);
app.use(NavBar);
app.use(Field);
app.use(Form);
app.use(List);
app.use(PullRefresh);
app.use(Card);
app.use(Tag);
app.use(VanImage);
app.use(Col);
app.use(Row);
app.use(Grid);
app.use(GridItem);
app.use(Swipe);
app.use(SwipeItem);
app.use(Divider);
app.use(Tab);
app.use(Tabs);
app.use(Skeleton);
app.use(Badge);

app.use(store).use(router).mount("#app");
