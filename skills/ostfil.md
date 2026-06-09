### Project: ostfil

#### 方法

* ostfil

- uuid
- size 文件原始尺寸
- type 文件类型
- end_time 最后一次打开时间
- md5 文件的 md5 值

* ostfil_save(ostfil* ot, const char* file_path)

- 参数

```

archive: 指向 ot 结构体的指针

file_path: 文件路径


```

- 返回值

```
最后生成的
* uuid


```


- 描述

```
接收到文件

1. 判断storage目录是否存在
2. 判断文件是否存在
3. 生成uuid
4. 在storage目录下创建对应的uuid目录
5. 根据判断生成封面图
6. 将文件拷贝到对应的uuid目录中，文件名为 original
7. 通过zlib库进行压缩，并保存为original.z文件
8. 生成ostfil结构体
9. 将ostfil结构体写入到对应的uuid目录中info.json文件中
10. 返回ostfil结构体

```

* ostfil_get(ostfil* ot, const char* uuid)
- 参数

```
uuidarchive: 指向 ot 结构体的指针

uuid: 文件的 uuid

```
- 返回值
```
ostfil结构体

```
- 描述

```
根据传入的 uuid 获取文件信息

1. 判断storage目录是否存在
2. 判断传入的 uuid 是否存在
3. 读取对应的uuid目录中的info.json文件
4. 将读取到的信息写入到传入的 ostfil 结构体中
5. 更新end_time读取时间
6. 判断original文件是否存在
7. 如果不存在，则通过original.z文件进行解压，并保存为original文件
8. 返回 ostfil 结构体

```

