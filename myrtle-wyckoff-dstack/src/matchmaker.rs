extern crate optimized_lob;
use optimized_lob::{
    order::OrderId, orderbook_manager::OrderBookManager, price::Price, quantity::Qty, utils::BookId,
};

// matches an order against the current orderbook state, performing executions where necessary
// returns a tuple of 4 values,
// - total qty executed
// - total volume of matches (this is the revenue for an ask and the cost for a bid)
// - new order id if one was created
// - vec of matched orders (if any)
// Notes:
// Attempts the following process:
// 1. Receive order from user
// 2. Check bids or asks and price levels to see what levels if any it crosses
// 3. Check levels to get quantities to see how many get cleared
// 4. Remove orders from all crossed levels and reduce from unfilled levels
// 5. Add order for any remaining size
//
// This is the expected entrypoint for all new user trades
pub fn match_order(
    manager: &mut OrderBookManager,
    book_id: BookId,
    price32: u32,
    qty: Qty,
    is_bid: bool,
) -> (Qty, Qty, Option<OrderId>, Vec<OrderId>) {
    let mut matched_orders = Vec::new();
    let mut remaining_qty = qty;
    let mut volume = Qty(0);
    let mut new_order_id = None;
    let price = Price::from_u32(price32, is_bid);

    if let Some(book) = manager.books.get_mut(book_id.value() as usize).unwrap() {
        let levels = if is_bid {
            &mut book.asks
        } else {
            &mut book.bids
        };

        // Get all crossed levels
        let mut crossed_levels = Vec::new();

        let mut i = levels.len();
        while i > 0 {
            i -= 1;
            let level = levels.get(i);

            if (is_bid && level.price() > price) || (!is_bid && level.price() < price) {
                break; // No more matching levels
            }
            crossed_levels.push(level.level_id());
        }
        let level_pool = book.level_pool.clone();
        let level_orders = book.level_orders.clone();
        // Execute orders on crossed levels
        for level_id in crossed_levels.iter() {
            let level = level_pool.get(*level_id).unwrap();
            let level_size = level.size();
            let level_price = level.price().absolute() as u32;
            let order_iter = level_orders.get(level_id).unwrap().iter();

            match level_size.cmp(&remaining_qty) {
                std::cmp::Ordering::Less => {
                    for order in order_iter {
                        manager.execute_order(*order, remaining_qty);
                        matched_orders.push(*order);
                    }
                    remaining_qty -= level_size;
                    volume += Qty(level_size.value() * level_price);
                }
                std::cmp::Ordering::Greater => {
                    volume += Qty(remaining_qty.value() * level_price);
                    for order in order_iter {
                        let old_order = manager.oid_map.get(*order).unwrap().clone();
                        manager.execute_order(*order, remaining_qty);
                        matched_orders.push(*order);
                        if old_order.qty() < remaining_qty {
                            remaining_qty -= old_order.qty();
                        } else {
                            remaining_qty = Qty(0);
                            break;
                        }
                    }
                }
                std::cmp::Ordering::Equal => {
                    for order in order_iter {
                        manager.execute_order(*order, remaining_qty);
                        matched_orders.push(*order);
                    }
                    remaining_qty = Qty(0);
                    volume += Qty(level_size.value() * level_price);
                    break;
                }
            }
        }
    }

    if remaining_qty.gt(&Qty(0)) {
        let order_id = manager.oid_map.next_id();
        manager.add_order(order_id, book_id, remaining_qty, price32, is_bid);
        new_order_id = Some(order_id);
    }
    (
        volume,
        Qty(qty.value() - remaining_qty.value()),
        new_order_id,
        matched_orders,
    )
}
