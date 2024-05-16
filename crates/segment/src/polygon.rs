use std::cmp::Ordering;
use std::collections::VecDeque;

// 定义一个结构体来存储点的坐标
#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

// 计算两点之间的距离
fn distance(p1: &Point, p2: &Point) -> f64 {
    ((p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)).sqrt()
}

// 计算点到直线的距离
fn point_to_line_distance(p: &Point, start: &Point, end: &Point) -> f64 {
    let numerator = (end.x - start.x) * (start.y - p.y) - (start.x - p.x) * (end.y - start.y);
    let denominator = (end.x - start.x).hypot(end.y - start.y);
    numerator.abs() / denominator
}

// 实现多边形拟合（简化轮廓）
fn simplify_polygon(mut points: VecDeque<Point>, epsilon: f64) -> Vec<Point> {
    if points.len() <= 2 {
        return points.into();
    }

    let first = points.pop_front().unwrap();
    let last = points.pop_back().unwrap();

    let mut max_dist = 0.0;
    let mut index = 0;

    // 找到最大偏移量对应的点
    for (i, &point) in points.iter().enumerate() {
        let dist = point_to_line_distance(&point, &first, &last);
        if dist > max_dist {
            max_dist = dist;
            index = i;
        }
    }

    // 如果最大偏移量小于阈值，直接连接首尾点
    if max_dist < epsilon {
        vec![first, last]
    } else {
        // 否则，递归简化
        let mut simplified_points = simplify_polygon(points.iter().take(index + 1).cloned().collect(), epsilon);
        simplified_points.pop(); // 移除重复的点
        simplified_points.extend(simplify_polygon(points.iter().skip(index).cloned().collect(), epsilon));
        [vec![first], simplified_points, vec![last]].concat()
    }
}


#[cfg(test)]
mod test {
    use std::collections::VecDeque;

    use super::{simplify_polygon, Point};


    #[test]
    fn test() {
        // 示例点集
        let mut points = VecDeque::from([
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 0.0 },
            Point { x: 10.0, y: 10.0 },
            Point { x: 5.0, y: 5.0 },
            Point { x: 0.0, y: 10.0 },
        ]);
    
        // 拟合多边形，这里设置一个示例的epsilon值
        let epsilon = 1.0;
        let simplified_polygon = simplify_polygon(points, epsilon);
    
        // 打印简化后的多边形点
        for point in simplified_polygon {
            println!("{:?}", point);
        }
    }
}