# Akt-Wxml

一个个人练习用，将wxml简单解析为jsx的rust demo

由于rust在安全条件下共享引用的处理极为困难，本demo采用了[红绿树](https://docs.microsoft.com/zh-cn/archive/blogs/ericlippert/persistence-facades-and-roslyns-red-green-trees)策略，即一棵树不可变的，另一棵树可变的形式