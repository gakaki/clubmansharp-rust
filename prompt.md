 请完全使用中文回答.接下来希望将clubmanSharp该程序修改为rust编程语言,记得使用anyhow                         
thisiserror来处理错误.使用slint ui 来进行ui设计,另外要加上支持多客户端多ip控制的功能         

 由于该游戏控制器使用的是dualshock,可以参考python                                                │
│   vgamepad库,重新用rust编写一个类似的库来控制游戏手柄操作
加上mac平台的支持

另外可以参考gt7 telementry python库来判断事件来获取当前游戏的状态,和赛道情况等

注意当前环境为mac ,由于claude code不支持windows,清先跳过windows上的编译