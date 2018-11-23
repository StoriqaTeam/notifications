-- init values for templates
INSERT INTO templates(name, data ) VALUES
('store_moderation_status_for_user', '<html>
  <head>
    <title>The moderation status of the store has changed</title>
  </head>
  <body>
    Store {{store_id}} status has been changed. You can view current store info on <a href="{{cluster_url}}/store/{{store_id}}">this page</a>.
  </body>
</html>'),
('base_product_moderation_status_for_user', '<html>
  <head>
    <title>The moderation status of the base product has changed</title>
  </head>
  <body>
  Base product {{base_product_id}} status has been changed. You can view current base product info on <a href="{{cluster_url}}/store/{{store_id}}/products/{{base_product_id}}">this page</a>.
  </body>
</html>
'),
('store_moderation_status_for_moderator', '<html>
  <head>
    <title>The moderation status of the store has changed</title>
  </head>
  <body>
  Store {{store_id}} status has been changed. You can view current store info on <a href="{{cluster_url}}/store/{{store_id}}">this page</a>.  
  </body>
</html>
'),
('base_product_moderation_status_for_moderator', '<html>
  <head>
    <title>The moderation status of the base product has changed</title>
  </head>
  <body>
    Base product {{store_id}} status has been changed. You can view current store info on <a href="{{cluster_url}}/store/{{store_id}}/products/{{base_product_id}}">this page</a>.  
  </body>
</html>
'),
;