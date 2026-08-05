以下是核实后确认无误的GM2通用系统专属信息（Universal SysEx）完整列表及格式：

| 功能 | SysEx 数据 (十六进制) | 说明与来源 |
| :--- | :--- | :--- |
| **GM2 System On** <br> (GM2 系统开启) | `F0 7E <设备ID> 09 03 F7` | 开启GM2模式并复位所有设置。 |
| **Master Volume** <br> (主音量) | `F0 7E <设备ID> 04 01 01 <音量LSB> <音量MSB> F7` | 设置设备整体音量，默认值为 `7F 7F`。 |
| **Master Fine Tuning** <br> (主微调音) | `F0 7E <设备ID> 04 03 01 <调谐LSB> <调谐MSB> F7` | 精细调音，默认 `40 00`（对应A440Hz）。 |
| **Master Coarse Tuning** <br> (主粗调音) | `F0 7E <设备ID> 04 02 01 <半音值> F7` | 范围±64半音（GM2要求至少±12），默认 `40`。 |
| **Reverb Parameters** <br> (混响参数) | `F0 7F <设备ID> 04 05 01 01 01 01 01 [pp] [vv] F7` | 控制全局混响效果器的参数，如类型(`pp=00`)和时间(`pp=01`)。 |
| **Chorus Parameters** <br> (合唱参数) | `F0 7F <设备ID> 04 05 01 01 01 01 02 [pp] [vv] F7` | 控制全局合唱效果器的参数，如类型(`pp=00`)、速率(`pp=01`)等。 |
| **Channel Pressure Destination** <br> (通道压力目标设定) | `F0 7F <设备ID> 09 01 0n [pp rr] ... F7` | 将通道压力信息映射到音高、滤波等特定参数。 |
| **Control Change Destination** <br> (控制变更目标设定) | `F0 7F <设备ID> 09 03 0n cc [pp rr] ... F7` | 将特定的MIDI控制器（CC）映射到音高、滤波等参数。 |
| **Scale/Octave Tuning Adjust** <br> (音阶/八度调音) | `F0 7E <设备ID> 08 01 <音符编号> <调整值> F7` | 对指定MIDI音符进行微调，默认 `40` 为不偏移。 |
| **Key-Based Instrument Controllers** <br> (基于键位的乐器控制器) | `F0 7F <设备ID> 0A 01 0n kk [nn vv] ... F7` | 单独调整鼓组中每个音符的音量(`nn=07`)、声像(`nn=0A`)等参数。 |

> **请注意**：表格中的 `<设备ID>` 用于指定目标设备，使用 `7F` 代表“所有设备”。

这些消息的格式和信息均可在MMA的官方文档中找到，例如`General MIDI Level 2`规范、`Key-Based Instrument Controllers`的CA-023文档，以及一些设备的数据列表中都有详细记载。