# hanzi-demo

显示中文字符的样例项目，但是，，，`zh-show`，`chinese-sample`名字太难听了...

但是`hanzi-demo`虽然好听，但是这里展示的东西太多了...要换掉变成`display-demo`吗？？？

因为16M放不下（图片、字体、布局...因为懒，就放弃了不少hhh），所以改为了条件编译，想看部分功能演示的，在default启用或者加上特性条件编译选项！

> cargo ecos flash -r -- -s -- --features cmd-text,cmd-cli,cmd-snake

其中，也可以指定display来选择演示/编译的目标平台：EBD-SIM、ECOS-SSC1:ST7735、ECOS-SSC1:8*8LED点阵等等 ...

```toml
# 也可以尝试，就不赘述了，因为有其他时期，所以，就不修缮build.rs了，要模拟环境就特性target设置为`target-ui-sim`，将build.rs、.cargo/*.toml先放到其他地方，然后`cargo run`...
# 如果是在板子上，就移回来，改回target-st7735，然后选择几个要看的（卡的要死），然后`cargo ecos ...`，ECOS-SSC1:8*8LED点阵照着st7735实现drawable接口即可，就先鸽了
embedded-icon = "0.0"
embedded-iconoir = "0.2"
embedded-layout = "0.4"
embedded-sprites = "0.2"
embedded-charts = { version = "0.3", default-features = false, features = [
    "animations",
    "bar",
        "basic-charts",
    "donut",
        "floating-point",
    "integer-math",
        "line",
    "micromath", "pie"
] }
embedded-canvas = { version = "0.3", features = ["alloc"] }
embedded-ui = "0.0"
embedded-plots = "0.2"
```
