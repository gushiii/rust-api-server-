SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

-- ----------------------------
-- Table structure for products
-- ----------------------------
DROP TABLE IF EXISTS `products`;
CREATE TABLE `products` (
  `product_uuid` varchar(64) NOT NULL,
  `title` varchar(100) NOT NULL,
  `price` int DEFAULT '0',
  `status` varchar(255) NOT NULL,
  PRIMARY KEY (`product_uuid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- ----------------------------
-- Records of products
-- ----------------------------
BEGIN;
INSERT INTO `products` VALUES ('pk_001', 'iPhone 15 Pro Max 256GB', 9999, 'active');
INSERT INTO `products` VALUES ('pk_002', 'Sony WH-1000XM5 无线降噪耳机', 2299, 'pending');
INSERT INTO `products` VALUES ('pk_003', 'Nike Air Max 90 运动鞋', 899, 'active');
INSERT INTO `products` VALUES ('pk_004', 'Logitech MX Master 3S 无线鼠标', 699, 'pending');
INSERT INTO `products` VALUES ('pk_005', 'Kindle Paperwhite 5 电子书阅读器', 1068, 'pending');
INSERT INTO `products` VALUES ('pk_006', 'Nintendo Switch OLED 游戏主机', 2299, 'pending');
INSERT INTO `products` VALUES ('pk_007', '星巴克经典陶瓷马克杯 400ml', 129, 'pending');
INSERT INTO `products` VALUES ('pk_008', '戴森 Dyson V12 吸尘器', 3999, 'pending');
INSERT INTO `products` VALUES ('pk_009', '优衣库 UNIQLO 男装纯棉短袖 T 恤', 99, 'active');
INSERT INTO `products` VALUES ('pk_010', '安克 Anker 20000mAh 移动电源', 199, 'pending');
INSERT INTO `products` VALUES ('pk_011', 'Le Labo Santal 33 檀香木香水 50ml', 1650, 'pending');
INSERT INTO `products` VALUES ('pk_012', '斐尔可 FILCO 圣手二代机械键盘', 1099, 'pending');
INSERT INTO `products` VALUES ('pk_013', 'Stanley 保温保冷吸管杯 1.2L', 348, 'pending');
INSERT INTO `products` VALUES ('pk_014', 'Bose SoundLink Flex 便携蓝牙音箱', 1099, 'pending');
INSERT INTO `products` VALUES ('pk_015', '无印良品 MUJI 超声波香薰机', 380, 'active');
INSERT INTO `products` VALUES ('pk_016', 'Nespresso 雀巢胶囊咖啡机 Vertuo', 1288, 'pending');
INSERT INTO `products` VALUES ('pk_017', 'Patagonia Torrentshell 3L 防水外套', 1399, 'pending');
INSERT INTO `products` VALUES ('pk_018', 'iPad Air 11 英寸 M2 芯片 128GB', 4799, 'pending');
INSERT INTO `products` VALUES ('pk_019', 'Steam Deck OLED 掌上游戏机 512GB', 4299, 'pending');
INSERT INTO `products` VALUES ('pk_020', '乐高 LEGO 机械组保时捷 911 积木', 1399, 'active');
COMMIT;

SET FOREIGN_KEY_CHECKS = 1;
